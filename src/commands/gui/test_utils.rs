//! Test utilities for GUI module
//!
//! This module provides common test fixtures and utilities.

#![allow(dead_code)]

use tokio::sync::broadcast;

use super::state::GuiAppState;

/// Create a test GuiAppState with default configuration
pub fn create_test_state() -> GuiAppState {
    let log_receiver = broadcast::channel(1).1;
    GuiAppState::new(log_receiver)
}
