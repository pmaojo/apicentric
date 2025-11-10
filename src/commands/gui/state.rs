//! GUI Application State
//!
//! This module defines the state for the `egui` application.

#![cfg(feature = "gui")]

pub struct GuiAppState {
    pub services: Vec<String>,
    pub logs: Vec<String>,
    pub ai_prompt: String,
}

impl GuiAppState {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            logs: Vec::new(),
            ai_prompt: "Generate a new service".to_string(),
        }
    }
}
