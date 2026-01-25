//! API Simulator module
//!
//! This module contains the core logic for the API simulator.

pub mod admin_server;
pub mod axios_client;
pub mod config;
pub mod lifecycle;
pub mod log;
pub mod manager;
pub mod marketplace;
pub mod mockoon;
pub mod openapi;
pub mod postman;
pub mod react_query;
pub mod react_view;
pub mod recording_proxy;
pub mod registry;
pub mod route_registry;
pub mod router;
pub mod scripting;
pub mod service;
#[cfg(feature = "bluetooth")]
pub mod bluetooth;
pub mod template;
pub mod typescript;
#[cfg(feature = "file-watch")]
pub mod watcher;
pub mod wiremock;

// Re-export common types
pub use config::{EndpointDefinition, ResponseDefinition, ServiceDefinition, SimulatorConfig};
pub use log::RequestLogEntry;
pub use manager::ApiSimulatorManager;
pub use registry::ServiceRegistry;
pub use router::RequestRouter;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChange {
    ServiceAdded(String),
    ServiceModified(String),
    ServiceRemoved(String),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulatorStatus {
    pub is_active: bool,
    pub services_count: usize,
    pub active_services: Vec<ServiceInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub port: u16,
    pub base_path: String,
    pub endpoints_count: usize,
    pub is_running: bool,
}
