//! Integration tests for the `mcp` command.

#![cfg(feature = "mcp")]

use std::fs;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn test_mcp_command_starts_server_and_responds_to_initialize() {
    let mut child = Command::new("cargo")
        .args(&["run", "--features", "mcp", "--bin", "apicentric", "--", "mcp", "--test"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let request = fs::read_to_string("tests/mcp_request.json").unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(request.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
    } // drop stdin to close it

    // Give the server a moment to process the request and respond.
    std::thread::sleep(Duration::from_millis(100));

    let mut stdout = child.stdout.take().unwrap();
    let mut response = String::new();
    stdout.read_to_string(&mut response).unwrap();

    assert!(response.contains("jsonrpc"));
    assert!(response.contains("result"));

    child.kill().unwrap();
}
