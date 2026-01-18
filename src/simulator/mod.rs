//! API Simulator module for Apicentric
//!
//! This module provides a comprehensive service simulation system that enables offline development
//! by serving locally-defined APIs through YAML configuration files.

pub mod admin_server;
<<<<<<< HEAD
pub mod axios_client;
=======
>>>>>>> origin/main
pub mod config;
pub mod lifecycle;
pub mod log;
pub mod manager;
<<<<<<< HEAD
pub mod marketplace;
=======
>>>>>>> origin/main
pub mod mockoon;
pub mod openapi;
pub mod postman;
pub mod react_query;
pub mod react_view;
pub mod recording_proxy;
pub mod registry;
pub mod route_registry;
pub mod router;
pub mod service;
pub mod template;
pub mod typescript;
pub mod watcher;
pub mod wiremock;

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
#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulatorStatus {
    pub is_active: bool,
    pub services_count: usize,
    pub active_services: Vec<ServiceInfo>,
}

/// Information about a service instance
#[derive(Debug, Clone, serde::Serialize)]
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
