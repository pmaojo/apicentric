//! Data models for authentication.
//!
//! This module provides data models for users, registration requests, login
//! requests, and authentication responses.

use serde::{Deserialize, Serialize};

/// A user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The user's ID.
    pub id: i64,
    /// The user's username.
    pub username: String,
    /// The user's password hash.
    pub password_hash: String,
    /// The time the user was created.
    pub created_at: String,
}

/// A request to register a new user.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// The username of the user to register.
    pub username: String,
    /// The password of the user to register.
    pub password: String,
}

/// A request to log in a user.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// The username of the user to log in.
    pub username: String,
    /// The password of the user to log in.
    pub password: String,
}

/// An authentication response.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// The JWT for the authenticated user.
    pub token: String,
}
