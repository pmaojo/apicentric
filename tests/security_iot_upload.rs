#![cfg(all(not(target_arch = "wasm32"), feature = "webui"))]

use apicentric::cloud::CloudServer;
use apicentric::simulator::{config::SimulatorConfig, ApiSimulatorManager};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_upload_security_vulnerability() {
    // Setup temporary directory for IoT files
    let temp_dir = tempfile::tempdir().unwrap();
    let iot_dir = temp_dir.path().join("iot");
    std::env::set_var("APICENTRIC_IOT_DIR", iot_dir.to_str().unwrap());

    // Start server on a specific port
    let port = 9099;

    // Create simulator manager
    let config = SimulatorConfig {
        services_dir: temp_dir.path().join("services"),
        db_path: temp_dir.path().join("db.sqlite"),
        ..Default::default()
    };
    let manager = ApiSimulatorManager::new(config);
    let server = CloudServer::new(manager);

    // Spawn server in background
    tokio::spawn(async move {
        server.start(port).await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(1000)).await;

    // Attempt to upload a malicious file (e.g. .sh script)
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(b"#!/bin/bash\necho 'hacked'".to_vec())
            .file_name("exploit.sh")
            .mime_str("application/x-sh")
            .unwrap(),
    );

    let response = client
        .post(format!("http://localhost:{}/api/iot/upload", port))
        .multipart(form)
        .send()
        .await
        .unwrap();

    // In the vulnerable state, this should return 200 OK
    // After fix, it should return 400 Bad Request
    let status = response.status();
    println!("Upload response status: {}", status);

    // Also try to upload a valid CSV
    let form_valid = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(b"timestamp,value\n1,10".to_vec())
            .file_name("data.csv")
            .mime_str("text/csv")
            .unwrap(),
    );

    let response_valid = client
        .post(format!("http://localhost:{}/api/iot/upload", port))
        .multipart(form_valid)
        .send()
        .await
        .unwrap();

    println!("Valid upload response status: {}", response_valid.status());

    // Check if the file was actually created on disk
    let exploit_path = iot_dir.join("exploit.sh");
    if exploit_path.exists() {
        println!("VULNERABILITY CONFIRMED: exploit.sh was created on disk!");
    } else {
        println!("exploit.sh was NOT created on disk.");
    }
}
