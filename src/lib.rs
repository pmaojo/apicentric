// Enhanced error handling and validation
pub mod app;
pub mod config;
pub mod context;
pub mod errors;
pub mod execution;
pub mod usecases;
pub mod utils;
pub mod validation;

// Domain layer (core business logic)
pub mod domain;

// Adapter layer (implementations of domain ports)
pub mod adapters {
    pub mod contract_repository;
    pub mod cypress;
    pub mod cypress_test_runner;
    pub mod git;
    pub mod http_client;
    pub mod junit;
    pub mod metrics;
    pub mod metrics_facade;
    pub mod ui_cli;
    pub mod test_runner_delegator;
    pub mod mock_server;
    pub mod npm_integration;
    pub mod report_sink;
    pub mod route_indexer;
    pub mod route_indexer_adapter;
    pub mod server_manager;
    pub mod service_spec_loader;
    pub mod watcher;

    pub use contract_repository::*;
    pub use cypress_test_runner::*;
    pub use http_client::*;
    pub use metrics_facade::*;
    pub use ui_cli::*;
    pub use test_runner_delegator::*;
    pub use report_sink::*;
    pub use route_indexer_adapter::*;
    pub use service_spec_loader::*;
}

// API Simulator module
pub mod simulator;

// CLI module
pub mod cli;
pub mod cli_ui;

// Re-export commonly used types
pub use config::PulseConfig;
pub use context::{Context, ContextBuilder};
pub use errors::{PulseError, PulseResult};
pub use execution::ExecutionContext;
pub use simulator::{ApiSimulatorManager, ServiceDefinition, SimulatorConfig};

// Re-export contract testing functionality
pub use domain::contract_testing::*;
pub use domain::contract_use_cases::*;

// Puertos (interfaces)
pub trait TestRunnerPort {
    /// Ejecuta un conjunto de specs y devuelve un vector con tuplas (spec, éxito/fallo, duración_ms, error_opcional, casos)
    fn run_specs(
        &self,
        specs: &[String],
        workers: usize,
        retries: u8,
        headless: bool,
    ) -> PulseResult<Vec<(String, bool, u128, Option<String>, Vec<crate::domain::entities::TestCaseResult>)>>;
}

// Reexports from infrastructure
pub mod infrastructure {
    pub use crate::adapters::*;
    pub use crate::app::generate_docs::generate_docs;
    pub use crate::app::setup_npm::setup_npm_scripts;
}
pub use infrastructure::*;

// Re-export test execution use cases
pub use usecases::test_execution::{run_all, run_impacted, watch};

// Re-export mock api facade (migrated from previous monolithic lib)
pub mod mock {
    pub use crate::adapters::mock_server::{load_spec, run_mock_server, MockApiSpec};
}
