use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::ValidationError;
use crate::validation::ConfigValidator;

pub mod validation;
pub mod repository;

pub use repository::{ConfigRepository, FileConfigRepository, init_config, load_config, save_config};

/// Main configuration structure for Apicentric
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApicentricConfig {
    // Core configuration
    pub cypress_config_path: String,
    pub base_url: String,
    pub specs_pattern: String,
    pub routes_dir: PathBuf,
    pub specs_dir: PathBuf,
    pub reports_dir: String,
    pub index_cache_path: PathBuf,
    pub default_timeout: u64,

    // Optional sections with defaults
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub execution: ExecutionConfig,

    #[serde(default)]
    pub npm: NpmConfig,

    /// AI generation configuration
    #[serde(default)]
    pub ai: Option<AiConfig>,

    // API Simulator configuration
    #[serde(default)]
    pub simulator: Option<crate::simulator::config::SimulatorConfig>,

    // Legacy sections (maintained for backward compatibility)
    #[serde(default)]
    pub testcase: Option<TestCaseConfig>,

    #[serde(default)]
    pub metrics: Option<MetricsConfig>,
}

/// Server management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub auto_start: bool,

    #[serde(default = "default_start_command")]
    pub start_command: String,

    #[serde(default = "default_startup_timeout")]
    pub startup_timeout_ms: u64,

    #[serde(default = "default_health_check_retries")]
    pub health_check_retries: u8,

    #[serde(default)]
    pub skip_health_check: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            start_command: default_start_command(),
            startup_timeout_ms: default_startup_timeout(),
            health_check_retries: default_health_check_retries(),
            skip_health_check: false,
        }
    }
}

/// Execution mode and behavior configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionConfig {
    #[serde(default = "default_execution_mode")]
    pub mode: ExecutionMode,

    #[serde(default = "default_continue_on_failure")]
    pub continue_on_failure: bool,

    #[serde(default)]
    pub dry_run: bool,

    #[serde(default)]
    pub verbose: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: default_execution_mode(),
            continue_on_failure: default_continue_on_failure(),
            dry_run: false,
            verbose: false,
        }
    }
}

/// Execution modes
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    CI,
    Development,
    Debug,
}

/// NPM integration configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NpmConfig {
    #[serde(default = "default_apicentric_script")]
    pub apicentric_script: String,

    #[serde(default = "default_apicentric_watch_script")]
    pub apicentric_watch_script: String,

    #[serde(default = "default_dev_script")]
    pub dev_script: String,
}

impl Default for NpmConfig {
    fn default() -> Self {
        Self {
            apicentric_script: default_apicentric_script(),
            apicentric_watch_script: default_apicentric_watch_script(),
            dev_script: default_dev_script(),
        }
    }
}

/// Configuration for AI assisted code generation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiConfig {
    /// Provider to use for text generation.
    pub provider: AiProviderKind,
    /// Path to the local model when using the `local` provider.
    #[serde(default)]
    pub model_path: Option<String>,
    /// API key when using the `openai` provider.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Optional model identifier for remote providers.
    #[serde(default)]
    pub model: Option<String>,
}

/// Supported AI providers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderKind {
    /// Use a local language model such as `llama-rs`.
    Local,
    /// Use the OpenAI API.
    Openai,
}

/// Legacy test case configuration (for backward compatibility)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCaseConfig {
    pub pattern: String,
    pub timeout: u64,
    pub retries: u8,
}

/// Legacy metrics configuration (for backward compatibility)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub sentry: Option<SentryConfig>,
    pub prometheus: Option<PrometheusConfig>,
    pub allure: Option<AllureConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SentryConfig {
    pub enabled: bool,
    pub dsn: String,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrometheusConfig {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllureConfig {
    pub enabled: bool,
    pub report_dir: String,
}

// Default value functions
fn default_start_command() -> String {
    "npm run dev".to_string()
}

fn default_startup_timeout() -> u64 {
    30000
}

fn default_health_check_retries() -> u8 {
    5
}

fn default_execution_mode() -> ExecutionMode {
    ExecutionMode::Development
}

fn default_continue_on_failure() -> bool {
    true
}

fn default_apicentric_script() -> String {
    "cargo run --manifest-path utils/apicentric/Cargo.toml --".to_string()
}

fn default_apicentric_watch_script() -> String {
    "cargo run --manifest-path utils/apicentric/Cargo.toml -- watch".to_string()
}

fn default_dev_script() -> String {
    "npm run dev".to_string()
}

/// Builder for [`ApicentricConfig`] enforcing defaults and validation.
pub struct ApicentricConfigBuilder {
    config: ApicentricConfig,
}

impl ApicentricConfig {
    /// Start building a [`ApicentricConfig`].
    pub fn builder() -> ApicentricConfigBuilder {
        ApicentricConfigBuilder::default()
    }
}

impl Default for ApicentricConfigBuilder {
    fn default() -> Self {
        Self {
            config: ApicentricConfig {
                cypress_config_path: "cypress.config.ts".to_string(),
                base_url: "http://localhost:5173".to_string(),
                specs_pattern: "app/routes/**/test/*.cy.ts".to_string(),
                routes_dir: PathBuf::from("app/routes"),
                specs_dir: PathBuf::from("app/routes"),
                reports_dir: "cypress/reports".to_string(),
                index_cache_path: PathBuf::from(".apicentric/route-index.json"),
                default_timeout: 30000,
                server: ServerConfig::default(),
                execution: ExecutionConfig::default(),
                npm: NpmConfig::default(),
                ai: None,
                simulator: Some(crate::simulator::config::SimulatorConfig::default_config()),
                testcase: None,
                metrics: None,
            },
        }
    }
}

impl ApicentricConfigBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    pub fn default_timeout(mut self, timeout: u64) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    pub fn routes_dir(mut self, path: PathBuf) -> Self {
        self.config.routes_dir = path;
        self
    }

    pub fn specs_dir(mut self, path: PathBuf) -> Self {
        self.config.specs_dir = path;
        self
    }

    pub fn index_cache_path(mut self, path: PathBuf) -> Self {
        self.config.index_cache_path = path;
        self
    }

    pub fn simulator_services_dir(mut self, path: PathBuf) -> Self {
        if let Some(ref mut sim) = self.config.simulator {
            sim.services_dir = path;
        }
        self
    }

    /// Finalize the builder returning a validated [`ApicentricConfig`].
    pub fn build(self) -> Result<ApicentricConfig, Vec<ValidationError>> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Generate a default configuration
pub fn generate_default_config() -> ApicentricConfig {
    ApicentricConfig::builder()
        .build()
        .expect("default configuration should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn builder_produces_valid_default_config() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("routes")).unwrap();
        std::fs::create_dir_all(tmp.path().join("specs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("services")).unwrap();
        let config = ApicentricConfig::builder()
            .routes_dir(tmp.path().join("routes"))
            .specs_dir(tmp.path().join("specs"))
            .index_cache_path(tmp.path().join("index.json"))
            .simulator_services_dir(tmp.path().join("services"))
            .build()
            .unwrap();
        assert_eq!(config.base_url, "http://localhost:5173");
        assert_eq!(config.execution.mode, ExecutionMode::Development);
    }

    #[test]
    fn builder_detects_invalid_url() {
        let result = ApicentricConfig::builder().base_url("not-a-url").build();
        assert!(result.is_err());
    }
}

