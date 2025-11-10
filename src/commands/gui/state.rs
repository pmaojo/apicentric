//! GUI Application State
//!
//! This module defines the state for the `egui` application.

#![cfg(feature = "gui")]

use tokio::sync::broadcast;

pub struct GuiAppState {
    pub services: Vec<String>,
    pub logs: Vec<String>,
    pub ai_prompt: String,
    pub log_receiver: broadcast::Receiver<apicentric::simulator::log::RequestLogEntry>,
    pub ai_generated_yaml: Option<String>,
}

impl GuiAppState {
    pub fn new(log_receiver: broadcast::Receiver<apicentric::simulator::log::RequestLogEntry>) -> Self {
        Self {
            services: Vec::new(),
            logs: Vec::new(),
            ai_prompt: "Generate a new service".to_string(),
            log_receiver,
            ai_generated_yaml: None,
        }
    }
}
