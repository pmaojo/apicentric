//! Integration tests for the `mcp` command.

#![cfg(feature = "mcp")]

use assert_cmd::Command;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use tempfile::{tempdir, NamedTempFile};

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

use std::process::{Command as StdCommand, Stdio};

#[test]
fn test_mcp_create_service_tool() {
    let request = fs::read_to_string("tests/mcp_create_request.json").unwrap();

    let temp_dir = tempdir().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();
    let services_dir_str = services_dir.to_str().unwrap().replace("\\", "\\\\");
    let db_path = temp_dir.path().join("apicentric.db");
    let db_path_str = db_path.to_str().unwrap().replace("\\", "\\\\");

    let mut config_file = NamedTempFile::new().unwrap();
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

    config_file.write_all(config_content.as_bytes()).unwrap();
    let config_path = config_file.path();

    // Ensure the binary is built with the mcp feature enabled
    let build_status = StdCommand::new("cargo")
        .args(["build", "--bin", "apicentric", "--features", "mcp"])
        .status()
        .expect("Failed to build apicentric binary");
    assert!(build_status.success(), "Failed to build apicentric binary");

    let cargo_bin = env!("CARGO_BIN_EXE_apicentric");
    let mut child = StdCommand::new(cargo_bin)
        .arg("--config")
        .arg(config_path)
        .arg("mcp")
        .arg("--test")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Consume stderr in a separate thread to prevent blocking and allow debugging
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for l in reader.lines().map_while(Result::ok) {
            println!("[MCP Stderr] {}", l);
        }
    });

    // Write request and flush
    stdin.write_all(request.as_bytes()).unwrap();
    stdin.flush().unwrap();

    // Do NOT close stdin immediately. Keep it open while we read the response.
    // The server will shut down when we close stdin or kill the process later.

    let mut reader = BufReader::new(stdout);
    let mut found = false;
    let mut line = String::new();
    let mut accumulated_response = String::new();

    // Read lines from stdout
    while let Ok(n) = reader.read_line(&mut line) {
        if n == 0 {
            break; // EOF
        }
        accumulated_response.push_str(&line);
        if line.contains("Service 'test-service' created") {
            found = true;
            break;
        }
        line.clear();
    }

    // Kill the child process
    let _ = child.kill();
    let _ = child.wait();

    if !found {
        println!("MCP Response (Accumulated): {}", accumulated_response);
    }

    assert!(
        found,
        "Did not find expected success message in MCP response"
    );
}
