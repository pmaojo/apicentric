// Enhanced error handling and validation
use wasm_bindgen::prelude::*;

pub mod app;
pub mod config;
pub mod context;
pub mod errors;
pub mod utils;
pub mod validation;
pub mod collab;
pub mod storage;
pub mod ai;

// Domain layer (core business logic)
pub mod domain;

// Contract modules (execution, scenarios, reporting)
pub mod contract;

// Adapter layer (implementations of domain ports)
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
pub mod simulator;

// CLI module
pub mod cli;
pub mod cli_ui;

// Re-export commonly used types
pub use config::PulseConfig;
pub use context::{Context, ContextBuilder, ExecutionContext};
pub use errors::{PulseError, PulseResult};
pub use simulator::{ApiSimulatorManager, ServiceDefinition, SimulatorConfig};

// Re-export contract testing functionality
pub use domain::contract_testing::*;
pub use domain::contract::*;

// Reexports from infrastructure
pub mod infrastructure {
    pub use crate::adapters::*;
    pub use crate::app::generate_docs::generate_docs;
    pub use crate::app::setup_npm::setup_npm_scripts;
}
pub use infrastructure::*;

// Re-export mock api facade (migrated from previous monolithic lib)
pub mod mock {
    pub use crate::adapters::mock_server::{load_spec, run_mock_server, MockApiSpec};
}

// Simple example function exposed to WebAssembly consumers
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
