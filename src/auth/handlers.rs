//! Axum handlers for authentication.
//!
//! This module provides handlers for registering, logging in, and authenticating
//! users.

use axum::{Json, extract::Extension, http::{StatusCode, HeaderMap, header}};
use serde_json::json;
use std::sync::Arc;
use crate::auth::repository::AuthRepository;
use crate::auth::password::{hash_password, verify_password};
use crate::auth::jwt::{generate_token, JwtKeys, validate_token};
use crate::auth::model::{RegisterRequest, LoginRequest, AuthResponse};

/// The state for the authentication handlers.
pub struct AuthState {
    /// The authentication repository.
    pub repo: Arc<AuthRepository>,
    /// The JWT keys.
    pub keys: JwtKeys,
}

/// Registers a new user.
///
/// # Arguments
///
/// * `state` - The authentication state.
/// * `payload` - The registration request.
///
/// # Returns
///
/// A `Result` containing an `AuthResponse` if the registration was successful,
/// or a rejection otherwise.
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

/// Logs a user in.
///
/// # Arguments
///
/// * `state` - The authentication state.
/// * `payload` - The login request.
///
/// # Returns
///
/// A `Result` containing an `AuthResponse` if the login was successful, or a
/// rejection otherwise.
pub async fn login(Extension(state): Extension<Arc<AuthState>>, Json(payload): Json<LoginRequest>) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = state.repo.find_by_username(&payload.username).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let Some(user) = user else { return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())); };
    let ok = verify_password(&user.password_hash, &payload.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok { return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())); }
    let token = generate_token(&payload.username, &state.keys, 24).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthResponse { token }))
}

/// A protected endpoint that returns the current user's claims.
///
/// # Arguments
///
/// * `state` - The authentication state.
/// * `headers` - The HTTP headers.
///
/// # Returns
///
/// A `Result` containing the user's claims if the request is authenticated,
/// or a rejection otherwise.
pub async fn me(Extension(state): Extension<Arc<AuthState>>, headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;
    let token = auth_header.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;
    let claims = validate_token(token, &state.keys).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;
    Ok(Json(json!({"username": claims.sub})))
}
