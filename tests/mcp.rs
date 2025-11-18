//! Integration tests for the `mcp` command.

#![cfg(feature = "mcp")]

use assert_cmd::Command;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_mcp_command_starts_server_and_responds_to_initialize() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let request = fs::read_to_string("tests/mcp_request.json").unwrap();
    temp_file.write_all(request.as_bytes()).unwrap();
    let temp_path = temp_file.path();

    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--features", "mcp", "--bin", "apicentric", "--", "mcp", "--test"]);
    cmd.pipe_stdin(temp_path).unwrap();

    let output = cmd.output().unwrap();
    let response = String::from_utf8(output.stdout).unwrap();

    assert!(response.contains("jsonrpc"));
    assert!(response.contains("result"));
}

use std::io::Write;
use std::process::{Command as StdCommand, Stdio};

#[test]
fn test_mcp_generate_and_start_service_tool() {
    let request = fs::read_to_string("tests/mcp_generate_request.json").unwrap();

    let mut config_file = NamedTempFile::new().unwrap();
    config_file
        .write_all(br#"{ "ai": { "provider": "openai", "api_key": "test-key" } }"#)
        .unwrap();
    let config_path = config_file.path();

    let cargo_bin = env!("CARGO_BIN_EXE_apicentric");
    let mut child = StdCommand::new(cargo_bin)
        .arg("--config")
        .arg(config_path)
        .arg("mcp")
        .arg("--test")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(request.as_bytes()).unwrap();
    } // stdin is closed here

    let output = child.wait_with_output().unwrap();
    let response = String::from_utf8(output.stdout).unwrap();

    assert!(response.contains("jsonrpc"));
    assert!(response.contains("result"));
    assert!(response.contains("Service"));
    assert!(response.contains("generated and started successfully"));
}
