use apicentric::cloud::CloudServer;
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig};
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn create_test_manager() -> Arc<ApiSimulatorManager> {
    let config = SimulatorConfig {
        enabled: true,
        services_dir: tempfile::tempdir().unwrap().path().to_path_buf(),
        ..Default::default()
    };
    Arc::new(ApiSimulatorManager::new(config))
}

#[tokio::test]
async fn test_health_check() {
    let manager = create_test_manager();
    let server = CloudServer::new(manager);
    let app = server.create_router();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_auth_endpoints() {
    let manager = create_test_manager();
    let server = CloudServer::new(manager);
    let app = server.create_router();

    // Register
    let register_body = serde_json::json!({
        "username": "testuser",
        "password": "password123"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_body).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Login
    let login_body = serde_json::json!({
        "username": "testuser",
        "password": "password123"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_body).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = body["token"].as_str().unwrap();

    // Me
    let request = Request::builder()
        .uri("/api/auth/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["username"], "testuser");
}

#[tokio::test]
async fn test_simulator_start_stop() {
    let manager = create_test_manager();
    let server = CloudServer::new(manager);
    let app = server.create_router();

    // Start
    let start_body = serde_json::json!({});
    let request = Request::builder()
        .method("POST")
        .uri("/api/simulator/start")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&start_body).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Stop
    let stop_body = serde_json::json!({});
    let request = Request::builder()
        .method("POST")
        .uri("/api/simulator/stop")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&stop_body).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
