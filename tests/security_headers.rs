#![cfg(not(target_arch = "wasm32"))]

use apicentric::cloud::CloudServer;
use apicentric::simulator::{
    config::{PortRange, SimulatorConfig},
    manager::ApiSimulatorManager,
};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_security_headers() {
    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = false;
    // Use random temp dir
    let services_dir = tempfile::tempdir().unwrap();
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10000,
        end: 10100,
    };

    let manager = ApiSimulatorManager::new(config);
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background on a random port
    let port = 10001;
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(port).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(1000)).await;

    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/health", port);

    // We try multiple times to avoid flakiness if server takes time to start
    let mut res = None;
    for _ in 0..5 {
        match client.get(&url).send().await {
            Ok(r) => {
                res = Some(r);
                break;
            }
            Err(_) => {
                sleep(Duration::from_millis(500)).await;
            }
        }
    }

    let res = res.expect("Failed to connect to server");

    assert_eq!(res.status(), 200);

    let headers = res.headers();
    println!("Headers received: {:?}", headers);

    // Check for security headers

    assert_eq!(
        headers.get("X-Content-Type-Options").and_then(|v| v.to_str().ok()),
        Some("nosniff"),
        "Missing or incorrect X-Content-Type-Options"
    );

    assert_eq!(
        headers.get("X-Frame-Options").and_then(|v| v.to_str().ok()),
        Some("SAMEORIGIN"),
        "Missing or incorrect X-Frame-Options"
    );

    assert_eq!(
        headers.get("X-XSS-Protection").and_then(|v| v.to_str().ok()),
        Some("1; mode=block"),
        "Missing or incorrect X-XSS-Protection"
    );

    assert_eq!(
        headers.get("Referrer-Policy").and_then(|v| v.to_str().ok()),
        Some("strict-origin-when-cross-origin"),
        "Missing or incorrect Referrer-Policy"
    );

    assert!(
        headers.contains_key("Content-Security-Policy"),
        "Missing Content-Security-Policy"
    );

    // Cleanup
    cloud_handle.abort();
}
