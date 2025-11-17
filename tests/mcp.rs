//! Integration tests for the `mcp` command.

#![cfg(feature = "mcp")]

use assert_cmd::Command;
use std::fs;
use std::io::{Read, Write};
use std::process::{Stdio};
use std::time::Duration;

#[test]
fn test_mcp_command_starts_server_and_responds_to_initialize() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    let mut child = cmd.arg("mcp")
        .pipe_stdin(Stdio::piped())
        .pipe_stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let request = fs::read_to_string("tests/mcp_request.json").unwrap();
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(request.as_bytes()).unwrap();

    // Give the server a moment to process the request and respond.
    std::thread::sleep(Duration::from_millis(100));

    let mut stdout = child.stdout.take().unwrap();
    let mut response = String::new();
    stdout.read_to_string(&mut response).unwrap();

    assert!(response.contains("jsonrpc"));
    assert!(response.contains("result"));

    child.kill().unwrap();
}
