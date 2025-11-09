//! The cloud API.
//!
//! This module provides a cloud API for managing services.

pub mod server;
pub mod api;
pub mod handlers;

pub use server::CloudServer;
