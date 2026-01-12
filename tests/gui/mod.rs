//! GUI Module Tests
//!
//! This module contains tests for the GUI functionality.
//! Tests are only compiled when the `gui` feature is enabled.

#![cfg(feature = "gui")]

mod ai_tests;
mod event_handler_tests;
mod log_tests;
mod message_tests;
mod mocks;
mod service_tests;
mod state_tests;

// Re-export mocks for use in other test modules
pub use mocks::*;
