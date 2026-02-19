#![doc = include_str!("../README.md")]

// Ensure that code within the library can refer to the crate as `apicentric`
extern crate self as apicentric;

// Enhanced error handling and validation
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod ai;
#[cfg(not(target_arch = "wasm32"))]
pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod auth;
#[cfg(all(not(target_arch = "wasm32"), feature = "webui"))]
pub mod cloud;
#[cfg(not(target_arch = "wasm32"))]
pub mod commands;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod context;
#[cfg(not(target_arch = "wasm32"))]
pub mod env_config;
#[cfg(not(target_arch = "wasm32"))]
pub mod errors;
#[cfg(not(target_arch = "wasm32"))]
pub mod logging;
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod utils;
#[cfg(not(target_arch = "wasm32"))]
pub mod validation;

// Domain layer (core business logic)
#[cfg(not(target_arch = "wasm32"))]
pub mod domain;

// Contract modules (execution, scenarios, reporting)
#[cfg(all(not(target_arch = "wasm32"), feature = "contract-testing"))]
pub mod contract;

// Adapter layer (implementations of domain ports)
#[cfg(not(target_arch = "wasm32"))]
pub mod adapters {
    #[cfg(feature = "contract-testing")]
    pub mod contract_repository;
    #[cfg(feature = "contract-testing")]
    pub mod http_client;
    #[cfg(feature = "contract-testing")]
    pub mod mock_server;
    #[cfg(feature = "contract-testing")]
    pub mod noop_telemetry;
    #[cfg(feature = "contract-testing")]
    pub mod report_sink;
    #[cfg(feature = "contract-testing")]
    pub mod service_spec_loader;
    #[cfg(feature = "contract-testing")]
    pub mod simulator_manager_adapter;
    pub mod ui_cli;

    #[cfg(feature = "contract-testing")]
    pub use contract_repository::*;
    #[cfg(feature = "contract-testing")]
    pub use http_client::*;
    #[cfg(feature = "contract-testing")]
    pub use noop_telemetry::*;
    #[cfg(feature = "contract-testing")]
    pub use report_sink::*;
    #[cfg(feature = "contract-testing")]
    pub use service_spec_loader::*;
    #[cfg(feature = "contract-testing")]
    pub use simulator_manager_adapter::*;
    pub use ui_cli::*;
}

// API Simulator module
#[cfg(not(target_arch = "wasm32"))]
pub mod simulator;

// CLI module
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(all(not(target_arch = "wasm32"), feature = "tui"))]
pub mod cli_ui;

// IoT module
#[cfg(all(not(target_arch = "wasm32"), feature = "iot"))]
pub mod iot;

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
#[cfg(all(not(target_arch = "wasm32"), feature = "contract-testing"))]
pub use domain::contract::*;
#[cfg(all(not(target_arch = "wasm32"), feature = "contract-testing"))]
pub use domain::contract_testing::*;

/// Re-exports from the infrastructure layer.
#[cfg(not(target_arch = "wasm32"))]
pub mod infrastructure {
    pub use crate::adapters::*;
}
#[cfg(not(target_arch = "wasm32"))]
pub use infrastructure::*;

/// Re-export of the mock API facade.
#[cfg(all(not(target_arch = "wasm32"), feature = "contract-testing"))]
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
