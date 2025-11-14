//! Authentication and authorization.
//!
//! This module provides everything needed for authentication and authorization,
//! including data models, password hashing, JWT handling, a repository for
//! storing user data, and Axum handlers for handling authentication requests.

pub mod model;
pub mod password;
pub mod jwt;
pub mod repository;
pub mod handlers;
pub mod extractor;
pub mod blacklist;
pub mod middleware;

pub use model::*;
pub use repository::*;
pub use handlers::*;
pub use extractor::*;
pub use blacklist::*;
pub use middleware::*;
