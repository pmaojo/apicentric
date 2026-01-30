#![cfg(all(not(target_arch = "wasm32"), feature = "webui"))]

use apicentric::cloud::CloudServer;
use apicentric::simulator::{config::SimulatorConfig, ApiSimulatorManager};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_upload_streaming_valid() {
    // Setup temporary directory for IoT files
    let temp_dir = tempfile::tempdir().unwrap();
    let iot_dir = temp_dir.path().join("iot");
    std::env::set_var("APICENTRIC_IOT_DIR", iot_dir.to_str().unwrap());

    // Start server on a specific port (different from other tests to avoid conflicts)
    let port = 9100;

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

    let client = reqwest::Client::new();

    // 1. Test valid CSV upload
    let csv_content = "timestamp,value\n1,10\n2,20\n3,30";
    let form_valid = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(csv_content.as_bytes().to_vec())
            .file_name("data_stream.csv")
            .mime_str("text/csv")
            .unwrap(),
    );

    let response_valid = client
        .post(format!("http://localhost:{}/api/iot/upload", port))
        .multipart(form_valid)
        .send()
        .await
        .unwrap();

    assert_eq!(response_valid.status(), 200);

    // Verify file content on disk
    let file_path = iot_dir.join("data_stream.csv");
    assert!(file_path.exists(), "File should exist on disk");

    let content = std::fs::read_to_string(file_path).unwrap();
    assert_eq!(content, csv_content, "File content should match uploaded content");

    println!("✅ Valid streaming upload confirmed");
}
