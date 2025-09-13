//! API Simulator module for Pulse
//!
//! This module provides a comprehensive service simulation system that enables offline development
//! by serving locally-defined APIs through YAML configuration files.

pub mod config;
pub mod log;
pub mod manager;
pub mod registry;
pub mod router;
pub mod service;
pub mod template;
pub mod watcher;
pub mod openapi;
pub mod mockoon;

#[cfg(test)]
mod integration_test;

// Re-export commonly used types
pub use config::{EndpointDefinition, ResponseDefinition, ServiceDefinition, SimulatorConfig};
pub use manager::ApiSimulatorManager;
pub use registry::ServiceRegistry;
pub use router::RequestRouter;
pub use service::ServiceInstance;
pub use template::{RequestContext, TemplateContext, TemplateEngine};

// Re-export for convenience, but not used in this module directly

/// Status information for the simulator
#[derive(Debug, Clone)]
pub struct SimulatorStatus {
    pub is_active: bool,
    pub services_count: usize,
    pub active_services: Vec<ServiceInfo>,
}

/// Information about a service instance
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub port: u16,
    pub base_path: String,
    pub endpoints_count: usize,
    pub is_running: bool,
}

/// Configuration change event for hot-reload functionality
#[derive(Debug, Clone)]
pub enum ConfigChange {
    ServiceAdded(String),
    ServiceModified(String),
    ServiceRemoved(String),
}
