use axum::{Json, extract::Extension, http::{StatusCode, HeaderMap, header}};
use serde_json::json;
use std::sync::Arc;
use crate::auth::repository::AuthRepository;
use crate::auth::password::{hash_password, verify_password};
use crate::auth::jwt::{generate_token, JwtKeys, validate_token};
use crate::auth::model::{RegisterRequest, LoginRequest, AuthResponse};

pub struct AuthState {
    pub repo: Arc<AuthRepository>,
    pub keys: JwtKeys,
}

pub async fn register(Extension(state): Extension<Arc<AuthState>>, Json(payload): Json<RegisterRequest>) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        return Err((StatusCode::BAD_REQUEST, "Invalid username or password (min length 6)".into()));
    }
    if let Ok(Some(_existing)) = state.repo.find_by_username(&payload.username) {
        return Err((StatusCode::CONFLICT, "Username already exists".into()));
    }
    let hash = hash_password(&payload.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.repo.create_user(&payload.username, &hash).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let token = generate_token(&payload.username, &state.keys, 24).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthResponse { token }))
}

pub async fn login(Extension(state): Extension<Arc<AuthState>>, Json(payload): Json<LoginRequest>) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = state.repo.find_by_username(&payload.username).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let Some(user) = user else { return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())); };
    let ok = verify_password(&user.password_hash, &payload.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok { return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())); }
    let token = generate_token(&payload.username, &state.keys, 24).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthResponse { token }))
}

// Simple extractor-like protected endpoint example (can be expanded later)
pub async fn me(Extension(state): Extension<Arc<AuthState>>, headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;
    let token = auth_header.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;
    let claims = validate_token(token, &state.keys).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;
    Ok(Json(json!({"username": claims.sub})))
}
