use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod server;
pub mod endpoint;
pub mod validation;

pub use server::{ServerConfig, CorsConfig};
pub use endpoint::{
    EndpointKind, StreamConfig, PeriodicMessage, EndpointDefinition,
    ParameterDefinition, ParameterLocation, RequestBodyDefinition, ResponseDefinition,
    SideEffect, ScenarioDefinition, ScenarioStrategy, ScenarioConditions, ScenarioResponse,
};
pub use validation::{ConfigLoader, LoadError, LoadErrorType, LoadResult, ValidationSummary};

fn default_db_path() -> PathBuf {
    PathBuf::from("pulse.db")
}

/// Main simulator configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulatorConfig {
    /// Whether the simulator is enabled
    pub enabled: bool,
    /// Directory containing service definition YAML files
    pub services_dir: PathBuf,
    /// Port range for automatic port assignment
    pub port_range: PortRange,
    /// Path to SQLite database for persistent storage
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
    /// Global behavior settings
    #[serde(default)]
    pub global_behavior: Option<BehaviorConfig>,
}

impl SimulatorConfig {
    /// Create a new simulator configuration with environment variable override
    pub fn new(enabled: bool, services_dir: PathBuf, port_range: PortRange) -> Self {
        Self {
            enabled: Self::check_environment_override(enabled),
            services_dir,
            port_range,
            db_path: default_db_path(),
            global_behavior: None,
        }
    }

    /// Create a default simulator configuration
    pub fn default_config() -> Self {
        Self {
            enabled: Self::check_environment_override(false),
            services_dir: PathBuf::from("services"),
            port_range: PortRange { start: 8000, end: 8999 },
            db_path: default_db_path(),
            global_behavior: None,
        }
    }

    /// Check if the simulator should be enabled based on environment variables
    /// Environment variable PULSE_API_SIMULATOR overrides configuration setting
    fn check_environment_override(config_enabled: bool) -> bool {
        match std::env::var("PULSE_API_SIMULATOR") {
            Ok(value) => {
                let normalized = value.to_lowercase();
                normalized == "true"
                    || normalized == "1"
                    || normalized == "yes"
                    || normalized == "on"
            }
            Err(_) => config_enabled,
        }
    }

    /// Check if the simulator is enabled (considering environment variables)
    pub fn is_enabled(&self) -> bool {
        Self::check_environment_override(self.enabled)
    }

    /// Get the effective enabled state with environment variable consideration
    pub fn effective_enabled_state(&self) -> (bool, bool) {
        let env_override = std::env::var("PULSE_API_SIMULATOR").is_ok();
        let effective_enabled = self.is_enabled();
        (effective_enabled, env_override)
    }
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            services_dir: PathBuf::from("services"),
            port_range: PortRange { start: 8000, end: 8999 },
            db_path: default_db_path(),
            global_behavior: None,
        }
    }
}

/// Port range configuration for service assignment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

/// Service definition loaded from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceDefinition {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub server: ServerConfig,
    pub models: Option<HashMap<String, serde_json::Value>>, // JSON Schema definitions
    pub fixtures: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub bucket: Option<HashMap<String, serde_json::Value>>,
    pub endpoints: Vec<EndpointDefinition>,
    #[serde(default)]
    pub graphql: Option<GraphQLConfig>,
    #[serde(default)]
    pub behavior: Option<BehaviorConfig>,
}

/// GraphQL configuration for a service
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQLConfig {
    /// Path to the GraphQL schema file (.graphql)
    pub schema_path: String,
    /// Map of operation names to Handlebars template files
    pub mocks: HashMap<String, String>,
}

/// Behavior configuration for simulation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BehaviorConfig {
    #[serde(default)]
    pub latency: Option<LatencyConfig>,
    #[serde(default)]
    pub error_simulation: Option<ErrorSimulationConfig>,
    #[serde(default)]
    pub rate_limiting: Option<RateLimitingConfig>,
}

/// Latency simulation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LatencyConfig {
    pub min_ms: u64,
    pub max_ms: u64,
}

/// Error simulation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorSimulationConfig {
    pub enabled: bool,
    pub rate: f64, // Error rate between 0.0 and 1.0
    pub status_codes: Option<Vec<u16>>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
}
