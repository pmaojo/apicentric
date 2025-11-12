//! Authentication middleware for protecting endpoints.
//!
//! This module provides middleware for validating JWT tokens on protected
//! endpoints with optional authentication based on configuration.

use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use std::sync::Arc;
use serde_json::json;
use crate::auth::{jwt::validate_token, handlers::AuthState};

/// Error response for authentication failures.
#[derive(serde::Serialize)]
pub struct AuthError {
    pub error: String,
    pub code: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": self.error,
                "code": self.code
            }))
        ).into_response()
    }
}

/// Middleware that requires a valid JWT token.
///
/// This middleware validates the JWT token from the Authorization header
/// and checks if it's blacklisted. If the token is invalid or missing,
/// it returns a 401 Unauthorized response.
///
/// # Arguments
///
/// * `auth_state` - The authentication state containing JWT keys and blacklist.
/// * `request` - The incoming HTTP request.
/// * `next` - The next middleware or handler in the chain.
///
/// # Returns
///
/// The response from the next handler if authentication succeeds, or an
/// error response if authentication fails.
pub async fn require_auth(
    auth_state: Arc<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AuthError {
            error: "Missing Authorization header".to_string(),
            code: "MISSING_AUTH_HEADER".to_string(),
        })?;

    // Extract Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError {
            error: "Invalid authorization scheme. Expected 'Bearer <token>'".to_string(),
            code: "INVALID_AUTH_SCHEME".to_string(),
        })?;

    // Check if token is blacklisted
    if auth_state.blacklist.is_blacklisted(token).await {
        return Err(AuthError {
            error: "Token has been revoked".to_string(),
            code: "TOKEN_REVOKED".to_string(),
        });
    }

    // Validate token
    let claims = validate_token(token, &auth_state.keys)
        .map_err(|e| {
            let (error, code) = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    ("Token has expired".to_string(), "TOKEN_EXPIRED".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    ("Invalid token format".to_string(), "INVALID_TOKEN".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    ("Invalid token signature".to_string(), "INVALID_SIGNATURE".to_string())
                }
                _ => {
                    ("Token validation failed".to_string(), "TOKEN_VALIDATION_FAILED".to_string())
                }
            };
            AuthError { error, code }
        })?;

    // Add claims to request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    // Continue to next handler
    Ok(next.run(request).await)
}

/// Middleware that optionally requires authentication based on configuration.
///
/// This middleware checks the `APICENTRIC_PROTECT_SERVICES` environment variable.
/// If set to "true" or "1", it requires authentication. Otherwise, it allows
/// all requests through without authentication.
///
/// # Arguments
///
/// * `auth_state` - The authentication state containing JWT keys and blacklist.
/// * `protect_services` - Whether to enforce authentication.
/// * `request` - The incoming HTTP request.
/// * `next` - The next middleware or handler in the chain.
///
/// # Returns
///
/// The response from the next handler, with optional authentication enforcement.
pub async fn optional_auth(
    auth_state: Arc<AuthState>,
    protect_services: bool,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    if protect_services {
        require_auth(auth_state, request, next).await
    } else {
        Ok(next.run(request).await)
    }
}

// Tests are commented out due to complexity with Axum test setup
// The middleware is tested through integration tests instead
// 
// #[cfg(test)]
// mod tests {
//     // Test implementation would go here
// }
