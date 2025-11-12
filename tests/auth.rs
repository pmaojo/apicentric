use apicentric::auth::{
    handlers::{login, register, AuthState},
    jwt::{generate_token, validate_token, JwtKeys},
    model::{LoginRequest, RegisterRequest},
    repository::AuthRepository,
};
use axum::{body::Body, http::{Request, StatusCode}, response::Json, extract::Extension};
use std::sync::Arc;
use tempfile::tempdir;

async fn setup() -> AuthState {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test-auth.db");
    let repo = AuthRepository::new(db_path).unwrap();
    let keys = JwtKeys::from_secret("test-secret");
    AuthState {
        repo: Arc::new(repo),
        keys,
    }
}

#[tokio::test]
async fn test_register_and_login() {
    let state = Arc::new(setup().await);
    let app = axum::Router::new()
        .route("/register", axum::routing::post(register))
        .route("/login", axum::routing::post(login))
        .layer(Extension(state.clone()));

    // Register a new user
    let register_payload = RegisterRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    use tower::ServiceExt;
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Log in with the new user
    let login_payload = LoginRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };
    let req = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_jwt_validation() {
    let keys = JwtKeys::from_secret("secret");
    let token = generate_token("user", &keys, 1).unwrap();
    let claims = validate_token(&token, &keys).unwrap();
    assert_eq!(claims.sub, "user");
}
