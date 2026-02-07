#![cfg(feature = "webui")]

use apicentric::cloud::handlers::{import_from_url, ImportUrlRequest};
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig};
use axum::extract::State;
use axum::Json;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_ssrf_prevention() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let services_dir = temp_dir.path().join("services");
    std::fs::create_dir(&services_dir).unwrap();

    // Create config
    let mut config = SimulatorConfig::default_config();
    config.services_dir = services_dir.clone();
    config.db_path = temp_dir.path().join("test.db");

    // Initialize simulator
    let simulator = Arc::new(ApiSimulatorManager::new(config));

    // Test Cases

    // 1. Localhost (IPv4)
    let request = ImportUrlRequest {
        url: "http://127.0.0.1:8080/sensitive".to_string(),
        format: None,
    };
    let result = import_from_url(State(simulator.clone()), Json(request)).await;

    match result {
        Ok(_) => panic!("Should have failed due to SSRF protection"),
        Err(e) => {
            // Check for forbidden status code (403)
            assert_eq!(e.status, axum::http::StatusCode::FORBIDDEN);
            assert!(e.response.message.contains("Security Error"));
        }
    }

    // 2. Localhost (IPv6)
    let request = ImportUrlRequest {
        url: "http://[::1]:8080/sensitive".to_string(),
        format: None,
    };
    let result = import_from_url(State(simulator.clone()), Json(request)).await;
    match result {
        Ok(_) => panic!("Should have failed due to SSRF protection"),
        Err(e) => {
            assert_eq!(e.status, axum::http::StatusCode::FORBIDDEN);
        }
    }

    // 3. Private Network
    let request = ImportUrlRequest {
        url: "http://192.168.1.1/router-admin".to_string(),
        format: None,
    };
    let result = import_from_url(State(simulator.clone()), Json(request)).await;
    match result {
        Ok(_) => panic!("Should have failed due to SSRF protection"),
        Err(e) => {
            assert_eq!(e.status, axum::http::StatusCode::FORBIDDEN);
        }
    }

    // 4. Cloud Metadata Service (AWS/GCP/Azure)
    let request = ImportUrlRequest {
        url: "http://169.254.169.254/latest/meta-data/".to_string(),
        format: None,
    };
    let result = import_from_url(State(simulator.clone()), Json(request)).await;
    match result {
        Ok(_) => panic!("Should have failed due to SSRF protection"),
        Err(e) => {
            assert_eq!(e.status, axum::http::StatusCode::FORBIDDEN);
        }
    }
}
