//! An extractor for authenticating users from JWTs.
//!
//! This module provides an `AuthUser` struct that can be used as an extractor
//! in Axum handlers to authenticate users from a JWT in the `Authorization`
//! header.

use axum::{async_trait, extract::FromRequestParts, http::{request::Parts, StatusCode}};
use crate::auth::jwt::{validate_token, Claims};
use axum::extract::Extension;
use std::sync::Arc;
use super::handlers::AuthState;

/// An extractor for authenticating users from JWTs.
pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    Extension<Arc<AuthState>>: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    /// Extracts a user's claims from a request.
    ///
    /// # Arguments
    ///
    /// * `parts` - The parts of the request.
    /// * `state` - The application state.
    ///
    /// # Returns
    ///
    /// A `Result` containing the user's claims if the request is authenticated,
    /// or a rejection otherwise.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Get auth state
        let Extension(auth_state) = Extension::<Arc<AuthState>>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing auth state".to_string()))?;
        // Read header
        let header = parts.headers.get(axum::http::header::AUTHORIZATION).and_then(|h| h.to_str().ok()).ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;
        let token = header.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "Invalid auth scheme".to_string()))?;
        let claims = validate_token(token, &auth_state.keys).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;
        Ok(AuthUser(claims))
    }
}
