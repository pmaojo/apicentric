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
