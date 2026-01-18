//! Axum handlers for authentication.
//!
//! This module provides handlers for registering, logging in, and authenticating
//! users.

<<<<<<< HEAD
use crate::auth::blacklist::TokenBlacklist;
use crate::auth::jwt::{generate_token, validate_token, JwtKeys};
use crate::auth::model::{AuthResponse, LoginRequest, RegisterRequest};
use crate::auth::password::{hash_password, verify_password};
use crate::auth::repository::AuthRepository;
use axum::{
    extract::Extension,
    http::{header, HeaderMap, StatusCode},
    Json,
};
use serde_json::json;
use std::sync::Arc;
=======
use axum::{Json, extract::Extension, http::{StatusCode, HeaderMap, header}};
use serde_json::json;
use std::sync::Arc;
use crate::auth::repository::AuthRepository;
use crate::auth::password::{hash_password, verify_password};
use crate::auth::jwt::{generate_token, JwtKeys, validate_token};
use crate::auth::model::{RegisterRequest, LoginRequest, AuthResponse};
>>>>>>> origin/main

/// The state for the authentication handlers.
pub struct AuthState {
    /// The authentication repository.
    pub repo: Arc<AuthRepository>,
    /// The JWT keys.
    pub keys: JwtKeys,
<<<<<<< HEAD
    /// The token blacklist for logout functionality.
    pub blacklist: TokenBlacklist,
=======
>>>>>>> origin/main
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
<<<<<<< HEAD
pub async fn register(
    Extension(state): Extension<Arc<AuthState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid username or password (min length 6)".into(),
        ));
    }
    if let Ok(Some(_existing)) = state.repo.find_by_username(payload.username.clone()).await {
        return Err((StatusCode::CONFLICT, "Username already exists".into()));
    }
    let hash = hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .repo
        .create_user(payload.username.clone(), hash)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let token = generate_token(&payload.username, &state.keys, 24)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
=======
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
>>>>>>> origin/main
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
<<<<<<< HEAD
pub async fn login(
    Extension(state): Extension<Arc<AuthState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = state
        .repo
        .find_by_username(payload.username.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let Some(user) = user else {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    };
    let ok = verify_password(&user.password_hash, &payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }
    let token = generate_token(&payload.username, &state.keys, 24)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
=======
pub async fn login(Extension(state): Extension<Arc<AuthState>>, Json(payload): Json<LoginRequest>) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = state.repo.find_by_username(&payload.username).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let Some(user) = user else { return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())); };
    let ok = verify_password(&user.password_hash, &payload.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok { return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())); }
    let token = generate_token(&payload.username, &state.keys, 24).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
>>>>>>> origin/main
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
<<<<<<< HEAD
pub async fn me(
    Extension(state): Extension<Arc<AuthState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization header".to_string(),
        ))?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;

    // Check if token is blacklisted
    if state.blacklist.is_blacklisted(token).await {
        return Err((StatusCode::UNAUTHORIZED, "Token has been revoked".into()));
    }

    let claims = validate_token(token, &state.keys)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;
    Ok(Json(json!({"username": claims.sub})))
}

/// Refreshes a JWT token.
///
/// # Arguments
///
/// * `state` - The authentication state.
/// * `headers` - The HTTP headers.
///
/// # Returns
///
/// A `Result` containing a new `AuthResponse` with a refreshed token if the
/// current token is valid, or a rejection otherwise.
pub async fn refresh(
    Extension(state): Extension<Arc<AuthState>>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization header".to_string(),
        ))?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;

    // Check if token is blacklisted
    if state.blacklist.is_blacklisted(token).await {
        return Err((StatusCode::UNAUTHORIZED, "Token has been revoked".into()));
    }

    // Validate the current token
    let claims = validate_token(token, &state.keys)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    // Generate a new token with the same username
    let new_token = generate_token(&claims.sub, &state.keys, 24)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Optionally blacklist the old token to prevent reuse
    state.blacklist.add(token).await;

    Ok(Json(AuthResponse { token: new_token }))
}

/// Logs out a user by invalidating their JWT token.
///
/// # Arguments
///
/// * `state` - The authentication state.
/// * `headers` - The HTTP headers.
///
/// # Returns
///
/// A `Result` containing a success message if the logout was successful,
/// or a rejection otherwise.
pub async fn logout(
    Extension(state): Extension<Arc<AuthState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization header".to_string(),
        ))?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;

    // Validate the token before blacklisting (to ensure it's a valid token)
    validate_token(token, &state.keys)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    // Add token to blacklist
    state.blacklist.add(token).await;

    Ok(Json(json!({
        "message": "Successfully logged out",
        "success": true
    })))
}
=======
pub async fn me(Extension(state): Extension<Arc<AuthState>>, headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;
    let token = auth_header.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;
    let claims = validate_token(token, &state.keys).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;
    Ok(Json(json!({"username": claims.sub})))
}
>>>>>>> origin/main
