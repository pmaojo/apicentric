use std::process::Command;
use std::time::{Duration, Instant};
use reqwest::blocking::Client;

#[test]
fn test_cloud_command_starts_server() {
    // Build the binary with the webui feature
    let mut build_cmd = Command::new("cargo");
    build_cmd.args(["build", "--features", "webui"]);
    let build_status = build_cmd.status().expect("Failed to build apicentric binary");
    assert!(build_status.success(), "Failed to build apicentric binary");

    // Run the cloud command in the background
    let mut cmd = Command::new("target/debug/apicentric");
    let mut child = cmd.arg("cloud").spawn().expect("Failed to start apicentric cloud");

    // Wait for the server to become available
    let client = Client::new();
    let start_time = Instant::now();
    let mut response = None;
    while start_time.elapsed() < Duration::from_secs(30) {
        match client.get("http://localhost:8080/health").send() {
            Ok(res) => {
                response = Some(res);
                break;
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }

    // Kill the server process regardless of the outcome
    child.kill().expect("Failed to kill child process");

    // Assert that the server responded successfully
    let response = response.expect("Server did not become available within 30 seconds");
    assert!(response.status().is_success(), "Health check endpoint did not return a success status");
}
