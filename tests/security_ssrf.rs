#![cfg(feature = "webui")]

use apicentric::cloud::handlers::{import_from_url, ImportUrlRequest};
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig};
use axum::extract::State;
use axum::Json;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_ssrf_blocking() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = SimulatorConfig::default_config();
    // Use unique temporary paths to avoid conflicts and verify handling of clean environments
    config.db_path = temp_dir.path().join("db.sqlite");
    config.services_dir = temp_dir.path().join("services");

    // Initialize the manager which creates necessary directories/DBs
    let manager = Arc::new(ApiSimulatorManager::new(config));

    // Test 1: Block Localhost
    // The validator should reject "localhost" immediately
    let request_localhost = ImportUrlRequest {
        url: "http://localhost:8080/spec.yaml".to_string(),
        format: None,
    };

    let result_localhost = import_from_url(State(manager.clone()), Json(request_localhost)).await;

    assert!(result_localhost.is_err(), "Should block localhost URL");
    let error_localhost = result_localhost.unwrap_err();
    let error_msg_localhost = error_localhost.response.message;
    assert!(
        error_msg_localhost.contains("localhost is forbidden")
            || error_msg_localhost.contains("Security check failed"),
        "Unexpected error message: {}",
        error_msg_localhost
    );

    // Test 2: Block Private IP (e.g., 192.168.x.x)
    // The validator should resolve this IP (which is just parsing) and reject it
    let request_private = ImportUrlRequest {
        url: "http://192.168.1.50/spec.yaml".to_string(),
        format: None,
    };

    let result_private = import_from_url(State(manager.clone()), Json(request_private)).await;

    assert!(result_private.is_err(), "Should block private IP URL");
    let error_private = result_private.unwrap_err();
    let error_msg_private = error_private.response.message;
    assert!(
        error_msg_private.contains("private IP")
            || error_msg_private.contains("Security check failed"),
        "Unexpected error message: {}",
        error_msg_private
    );

    // Test 3: Block Link-Local IP (169.254.x.x)
    let request_link_local = ImportUrlRequest {
        url: "http://169.254.169.254/latest/meta-data".to_string(),
        format: None,
    };

    let result_link_local = import_from_url(State(manager.clone()), Json(request_link_local)).await;

    assert!(result_link_local.is_err(), "Should block link-local IP URL");
    let error_link_local = result_link_local.unwrap_err();
    let error_msg_link_local = error_link_local.response.message;
    assert!(
        error_msg_link_local.contains("private IP")
            || error_msg_link_local.contains("Security check failed"),
        "Unexpected error message: {}",
        error_msg_link_local
    );
}
