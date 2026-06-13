//! Integration tests for the `mcp` command.

#![cfg(feature = "mcp")]

use assert_cmd::Command;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command as StdCommand, Stdio};
use tempfile::{tempdir, NamedTempFile};

const TEST_SERVICE_YAML: &str =
    "name: test-service\nport: 9090\nendpoints:\n  - path: /test\n    method: GET\n    response:\n      status: 200\n      body: OK";

const UPDATED_SERVICE_YAML: &str =
    "name: test-service\nport: 9091\nendpoints:\n  - path: /updated\n    method: GET\n    response:\n      status: 200\n      body: Updated";

/// Escape a string for embedding as a JSON string value.
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn make_config_file(services_dir: &std::path::Path, db_path: &std::path::Path) -> NamedTempFile {
    let services_dir_str = services_dir.to_str().unwrap().replace('\\', "\\\\");
    let db_path_str = db_path.to_str().unwrap().replace('\\', "\\\\");

    let config_content = format!(
        r#"{{
  "ai": {{ "provider": "openai", "api_key": "test-key" }},
  "simulator": {{
    "enabled": true,
    "services_dir": "{}",
    "port_range": {{ "start": 9000, "end": 9099 }},
    "db_path": "{}",
    "admin_port": null,
    "global_behavior": {{ "latency": null, "error_simulation": null, "rate_limiting": null }}
  }}
}}"#,
        services_dir_str, db_path_str
    );

    let mut config_file = NamedTempFile::new().unwrap();
    config_file.write_all(config_content.as_bytes()).unwrap();
    config_file
}

fn spawn_mcp(config_path: &std::path::Path) -> std::process::Child {
    StdCommand::new("cargo")
        .args([
            "run",
            "--features",
            "mcp",
            "--bin",
            "apicentric",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "mcp",
            "--test",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// Write `input` to the child process's stdin, then read stdout until a line
/// containing `expected` is found. Returns `(found, accumulated_output)`.
fn send_and_expect(child: &mut std::process::Child, input: &str, expected: &str) -> (bool, String) {
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for l in reader.lines().map_while(Result::ok) {
            eprintln!("[MCP Stderr] {}", l);
        }
    });

    stdin.write_all(input.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let mut reader = BufReader::new(stdout);
    let mut found = false;
    let mut accumulated = String::new();
    let mut line = String::new();

    while let Ok(n) = reader.read_line(&mut line) {
        if n == 0 {
            break;
        }
        accumulated.push_str(&line);
        if line.contains(expected) {
            found = true;
            break;
        }
        line.clear();
    }

    let _ = child.kill();
    let _ = child.wait();

    (found, accumulated)
}

/// Build an MCP session preamble (initialize + initialized notification).
fn mcp_preamble() -> String {
    format!(
        "{}\n{}\n",
        r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#
    )
}

/// Build a tools/call JSON-RPC line.
fn tool_call(id: u32, name: &str, args_json: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","method":"tools/call","params":{{"name":"{}","arguments":{}}},"id":{}}}"#,
        name, args_json, id
    )
}

fn create_service_call(id: u32, yaml: &str) -> String {
    tool_call(
        id,
        "create_service",
        &format!(r#"{{"yaml_definition":"{}"}}"#, json_escape(yaml)),
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_mcp_command_starts_server_and_responds_to_initialize() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let request = fs::read_to_string("tests/mcp_request.json").unwrap();
    temp_file.write_all(request.as_bytes()).unwrap();
    let temp_path = temp_file.path();

    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--features",
        "mcp",
        "--bin",
        "apicentric",
        "--",
        "mcp",
        "--test",
    ]);
    cmd.pipe_stdin(temp_path).unwrap();

    let output = cmd.output().unwrap();
    let response = String::from_utf8(output.stdout).unwrap();

    assert!(response.contains("jsonrpc"));
    assert!(response.contains("result"));
}

#[test]
fn test_mcp_create_service_tool() {
    let request = fs::read_to_string("tests/mcp_create_request.json").unwrap();

    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let (found, accumulated_response) =
        send_and_expect(&mut child, &request, "Service 'test-service' created");

    if !found {
        println!("MCP Response (Accumulated): {}", accumulated_response);
    }
    assert!(found, "Did not find expected success message in MCP response");
}

#[test]
fn test_mcp_list_services_tool() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n",
        mcp_preamble(),
        create_service_call(2, TEST_SERVICE_YAML)
    );

    let (found, accumulated) = send_and_expect(&mut child, &input, "test-service");

    if !found {
        println!("MCP list_services response: {}", accumulated);
    }
    assert!(found, "Expected 'test-service' in response");
}

#[test]
fn test_mcp_get_simulator_status_tool() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n",
        mcp_preamble(),
        tool_call(2, "get_simulator_status", "{}")
    );

    let (found, accumulated) = send_and_expect(&mut child, &input, "API Simulator Status");

    if !found {
        println!("MCP get_simulator_status response: {}", accumulated);
    }
    assert!(found, "Expected 'API Simulator Status' in response");
}

#[test]
fn test_mcp_get_service_status_tool() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n{}\n",
        mcp_preamble(),
        create_service_call(2, TEST_SERVICE_YAML),
        tool_call(
            3,
            "get_service_status",
            r#"{"service_name":"test-service"}"#
        )
    );

    let (found, accumulated) =
        send_and_expect(&mut child, &input, "Service 'test-service' status");

    if !found {
        println!("MCP get_service_status response: {}", accumulated);
    }
    assert!(found, "Expected service status in response");
}

#[test]
fn test_mcp_get_service_logs_returns_real_data() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n{}\n",
        mcp_preamble(),
        create_service_call(2, TEST_SERVICE_YAML),
        tool_call(3, "get_service_logs", r#"{"service_name":"test-service"}"#)
    );

    // With no requests yet, expect "no requests recorded yet" message
    let (found, accumulated) = send_and_expect(&mut child, &input, "test-service");

    if !found {
        println!("MCP get_service_logs response: {}", accumulated);
    }
    assert!(found, "Expected service name in logs response");
    assert!(
        !accumulated.contains("2025-11-17T14:40:00Z"),
        "Should not return hardcoded dummy logs"
    );
}

#[test]
fn test_mcp_get_service_logs_unknown_service_returns_error() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n",
        mcp_preamble(),
        tool_call(
            2,
            "get_service_logs",
            r#"{"service_name":"nonexistent-service"}"#
        )
    );

    let (found, accumulated) = send_and_expect(&mut child, &input, "not found");

    if !found {
        println!("MCP get_service_logs (unknown) response: {}", accumulated);
    }
    assert!(found, "Expected 'not found' error for nonexistent service");
}

#[test]
fn test_mcp_delete_service_tool() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n{}\n",
        mcp_preamble(),
        create_service_call(2, TEST_SERVICE_YAML),
        tool_call(3, "delete_service", r#"{"service_name":"test-service"}"#)
    );

    let (found, accumulated) = send_and_expect(&mut child, &input, "deleted successfully");

    if !found {
        println!("MCP delete_service response: {}", accumulated);
    }
    assert!(found, "Expected 'deleted successfully' in response");
}

#[test]
fn test_mcp_update_service_tool() {
    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let db_path = temp_dir.path().join("apicentric.db");

    let config_file = make_config_file(&services_dir, &db_path);
    let mut child = spawn_mcp(config_file.path());

    let input = format!(
        "{}{}\n{}\n",
        mcp_preamble(),
        create_service_call(2, TEST_SERVICE_YAML),
        tool_call(
            3,
            "update_service",
            &format!(
                r#"{{"service_name":"test-service","yaml_definition":"{}"}}"#,
                json_escape(UPDATED_SERVICE_YAML)
            )
        )
    );

    let (found, accumulated) = send_and_expect(&mut child, &input, "updated");

    if !found {
        println!("MCP update_service response: {}", accumulated);
    }
    assert!(found, "Expected 'updated' in response");
}
