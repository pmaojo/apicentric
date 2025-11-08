// Enhanced error handling and validation
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod context;
#[cfg(not(target_arch = "wasm32"))]
pub mod errors;
#[cfg(not(target_arch = "wasm32"))]
pub mod utils;
#[cfg(not(target_arch = "wasm32"))]
pub mod validation;
#[cfg(not(target_arch = "wasm32"))]
pub mod collab;
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod ai;
#[cfg(not(target_arch = "wasm32"))]
pub mod cloud;
#[cfg(not(target_arch = "wasm32"))]
pub mod auth;

// Domain layer (core business logic)
#[cfg(not(target_arch = "wasm32"))]
pub mod domain;

// Contract modules (execution, scenarios, reporting)
#[cfg(not(target_arch = "wasm32"))]
pub mod contract;

// Adapter layer (implementations of domain ports)
#[cfg(not(target_arch = "wasm32"))]
pub mod adapters {
    pub mod contract_repository;
    pub mod http_client;
    pub mod ui_cli;
    pub mod mock_server;
    pub mod npm;
    pub mod report_sink;
    pub mod service_spec_loader;

    pub use contract_repository::*;
    pub use http_client::*;
    pub use ui_cli::*;
    pub use report_sink::*;
    pub use service_spec_loader::*;
}

// API Simulator module
#[cfg(not(target_arch = "wasm32"))]
pub mod simulator;

// CLI module
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli_ui;

// Re-export commonly used types
#[cfg(not(target_arch = "wasm32"))]
pub use config::ApicentricConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use context::{Context, ContextBuilder, ExecutionContext};
#[cfg(not(target_arch = "wasm32"))]
pub use errors::{ApicentricError, ApicentricResult};
#[cfg(not(target_arch = "wasm32"))]
pub use simulator::{ApiSimulatorManager, ServiceDefinition, SimulatorConfig};

// Re-export contract testing functionality
#[cfg(not(target_arch = "wasm32"))]
pub use domain::contract_testing::*;
#[cfg(not(target_arch = "wasm32"))]
pub use domain::contract::*;

// Reexports from infrastructure
#[cfg(not(target_arch = "wasm32"))]
pub mod infrastructure {
    pub use crate::adapters::*;
    pub use crate::app::setup_npm::setup_npm_scripts;
}
#[cfg(not(target_arch = "wasm32"))]
pub use infrastructure::*;

// Re-export mock api facade (migrated from previous monolithic lib)
#[cfg(not(target_arch = "wasm32"))]
pub mod mock {
    pub use crate::adapters::mock_server::{load_spec, run_mock_server, MockApiSpec};
}

// Simple example function exposed to WebAssembly consumers
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
