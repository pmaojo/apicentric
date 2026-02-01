#![cfg(all(not(target_arch = "wasm32"), feature = "webui"))]

use apicentric::cloud::iot_handlers;
use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use tower::Service;

// Use a static mutex to prevent race conditions when setting the environment variable
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[tokio::test]
async fn test_upload_scenarios() {
    // Acquire lock to ensure exclusive access to env var
    let _guard = ENV_LOCK.lock().unwrap();

    // --- Scenario 1: Upload Size Limit ---
    {
        // Setup temporary directory
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("APICENTRIC_IOT_DIR", temp_dir.path().to_str().unwrap());

        // Create a router with the handler, disable default body limit
        let mut app = Router::new()
            .route("/upload", post(iot_handlers::upload_replay_data))
            .layer(DefaultBodyLimit::disable());

        // Create a large body (11MB)
        let large_data = vec![0u8; 11 * 1024 * 1024];
        let boundary = "boundary";
        let body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"large.csv\"\r\nContent-Type: text/csv\r\n\r\n",
            boundary
        )
        .into_bytes();

        let mut full_body = body;
        full_body.extend(large_data);
        full_body.extend(format!("\r\n--{}--\r\n", boundary).into_bytes());

        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(full_body))
            .unwrap();

        let response = app.call(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Check response body for error message
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("File size exceeds 10MB limit"));
    }

    // --- Scenario 2: Valid Upload Size ---
    {
        // Setup temporary directory (new one)
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("APICENTRIC_IOT_DIR", temp_dir.path().to_str().unwrap());

        // Create a router with the handler
        let mut app = Router::new().route("/upload", post(iot_handlers::upload_replay_data));

        // Create a small body (1KB)
        let data = vec![b'a'; 1024];
        let boundary = "boundary";
        let body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"valid.csv\"\r\nContent-Type: text/csv\r\n\r\n",
            boundary
        )
        .into_bytes();

        let mut full_body = body;
        full_body.extend(data);
        full_body.extend(format!("\r\n--{}--\r\n", boundary).into_bytes());

        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(full_body))
            .unwrap();

        let response = app.call(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify file exists
        let file_path = temp_dir.path().join("valid.csv");
        assert!(file_path.exists());
        assert_eq!(std::fs::metadata(file_path).unwrap().len(), 1024);
    }
}
