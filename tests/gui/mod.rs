//! GUI Module Tests
//!
//! This module contains tests for the GUI functionality.
//! Tests are only compiled when the `gui` feature is enabled.

#![cfg(feature = "gui")]

mod mocks;
mod state_tests;
mod message_tests;
mod ai_tests;
mod event_handler_tests;
mod service_tests;
mod log_tests;

// Re-export mocks for use in other test modules
pub use mocks::*;
