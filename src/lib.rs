//! # Apicentric
//!
//! Apicentric is a powerful CLI tool and API simulator platform for developers.
//! This library contains the core functionality of Apicentric, including the API
//! simulator, contract testing, and code generation.
//!
//! The library is structured into several modules, each responsible for a specific
//! part of the application:
//!
//! - `app`: The main application entry point and setup.
//! - `config`: Application configuration management.
//! - `context`: The application context, which holds the configuration and other
//!   shared state.
//! - `errors`: Error handling and custom error types.
//! - `utils`: Utility functions.
//! - `validation`: Data validation.
//! - `storage`: Storage-related functionality.
//! - `ai`: AI-powered features.
//! - `domain`: The core business logic of the application.
//! - `contract`: Contract testing functionality.
//! - `adapters`: Implementations of the domain ports.
//! - `simulator`: The API simulator.
//! - `cli`: The command-line interface.
//! - `cli_ui`: The command-line user interface.

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
pub mod logging;
#[cfg(not(target_arch = "wasm32"))]
pub mod utils;
#[cfg(not(target_arch = "wasm32"))]
pub mod validation;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p"))]
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

/// Re-exports from the infrastructure layer.
#[cfg(not(target_arch = "wasm32"))]
pub mod infrastructure {
    pub use crate::adapters::*;
}
#[cfg(not(target_arch = "wasm32"))]
pub use infrastructure::*;

/// Re-export of the mock API facade.
#[cfg(not(target_arch = "wasm32"))]
pub mod mock {
    pub use crate::adapters::mock_server::{load_spec, run_mock_server, MockApiSpec};
}

/// A simple example function exposed to WebAssembly consumers.
///
/// # Arguments
///
/// * `name` - The name to greet.
///
/// # Returns
///
/// A greeting string.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
