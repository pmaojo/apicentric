//! Authentication and authorization.
//!
//! This module provides everything needed for authentication and authorization,
//! including data models, password hashing, JWT handling, a repository for
//! storing user data, and Axum handlers for handling authentication requests.

<<<<<<< HEAD
pub mod blacklist;
pub mod extractor;
pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod model;
pub mod password;
pub mod repository;

pub use blacklist::*;
pub use extractor::*;
pub use handlers::*;
pub use middleware::*;
pub use model::*;
pub use repository::*;
=======
pub mod model;
pub mod password;
pub mod jwt;
pub mod repository;
pub mod handlers;
pub mod extractor;

pub use model::*;
pub use repository::*;
pub use handlers::*;
pub use extractor::*;
>>>>>>> origin/main
