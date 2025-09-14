use crate::errors::{PulseError, PulseResult, ValidationError};
use crate::validation::{ConfigValidator, ValidationUtils};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure for Pulse
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PulseConfig {
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
    #[serde(default = "default_pulse_script")]
    pub pulse_script: String,

    #[serde(default = "default_pulse_watch_script")]
    pub pulse_watch_script: String,

    #[serde(default = "default_dev_script")]
    pub dev_script: String,
}

impl Default for NpmConfig {
    fn default() -> Self {
        Self {
            pulse_script: default_pulse_script(),
            pulse_watch_script: default_pulse_watch_script(),
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

impl ConfigValidator for AiConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        match self.provider {
            AiProviderKind::Local => {
                if self.model_path.as_deref().unwrap_or("").is_empty() {
                    errors.push(
                        ValidationError::new("ai.model_path", "model_path is required for local provider"),
                    );
                }
            }
            AiProviderKind::Openai => {
                if self.api_key.as_deref().unwrap_or("").is_empty() {
                    errors.push(
                        ValidationError::new("ai.api_key", "api_key is required for openai provider"),
                    );
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
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

fn default_pulse_script() -> String {
    "cargo run --manifest-path utils/mockforge/Cargo.toml --".to_string()
}

fn default_pulse_watch_script() -> String {
    "cargo run --manifest-path utils/mockforge/Cargo.toml -- watch".to_string()
}

fn default_dev_script() -> String {
    "npm run dev".to_string()
}

impl ConfigValidator for PulseConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate base URL
        if let Err(e) = ValidationUtils::validate_url(&self.base_url, "base_url") {
            errors.push(e);
        }

        // Validate non-empty strings
        if let Err(e) = ValidationUtils::validate_non_empty_string(
            &self.cypress_config_path,
            "cypress_config_path",
        ) {
            errors.push(e);
        }

        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.specs_pattern, "specs_pattern")
        {
            errors.push(e);
        }

        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.reports_dir, "reports_dir")
        {
            errors.push(e);
        }

        // Validate glob pattern
        if let Err(e) = ValidationUtils::validate_glob_pattern(&self.specs_pattern, "specs_pattern")
        {
            errors.push(e);
        }

        // Validate directories (create if they don't exist)
        if let Err(e) = ValidationUtils::validate_directory(&self.routes_dir, "routes_dir", true) {
            errors.push(e);
        }

        if let Err(e) = ValidationUtils::validate_directory(&self.specs_dir, "specs_dir", true) {
            errors.push(e);
        }

        // Validate parent directory for cache path
        if let Err(e) =
            ValidationUtils::validate_parent_directory(&self.index_cache_path, "index_cache_path")
        {
            errors.push(e);
        }

        // Validate numeric ranges
        if let Err(e) = ValidationUtils::validate_numeric_range(
            self.default_timeout,
            1000,
            300000,
            "default_timeout",
        ) {
            errors.push(e);
        }

        // Validate server config
        if let Err(mut server_errors) = self.server.validate() {
            errors.append(&mut server_errors);
        }

        // Validate execution config
        if let Err(mut exec_errors) = self.execution.validate() {
            errors.append(&mut exec_errors);
        }

        // Validate npm config
        if let Err(mut npm_errors) = self.npm.validate() {
            errors.append(&mut npm_errors);
        }

        // Validate AI config if present
        if let Some(ref ai) = self.ai {
            if let Err(mut ai_errors) = ai.validate() {
                errors.append(&mut ai_errors);
            }
        }

        // Validate simulator config if present
        if let Some(ref simulator) = self.simulator {
            if let Err(mut simulator_errors) = simulator.validate() {
                errors.append(&mut simulator_errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for ServerConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate start command is not empty
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.start_command, "server.start_command")
        {
            errors.push(e);
        }

        // Validate timeout range
        if let Err(e) = ValidationUtils::validate_numeric_range(
            self.startup_timeout_ms,
            5000,
            120000,
            "server.startup_timeout_ms",
        ) {
            errors.push(e);
        }

        // Validate retry count
        if let Err(e) = ValidationUtils::validate_numeric_range(
            self.health_check_retries,
            1,
            20,
            "server.health_check_retries",
        ) {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for ExecutionConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        // Execution config validation is mostly structural, no specific validation needed
        Ok(())
    }
}

impl ConfigValidator for NpmConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate scripts are not empty
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.pulse_script, "npm.pulse_script")
        {
            errors.push(e);
        }

        if let Err(e) = ValidationUtils::validate_non_empty_string(
            &self.pulse_watch_script,
            "npm.pulse_watch_script",
        ) {
            errors.push(e);
        }

        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.dev_script, "npm.dev_script")
        {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Load and validate configuration from file
pub fn load_config(path: &Path) -> PulseResult<PulseConfig> {
    // Check if config file exists
    if !path.exists() {
        return Err(PulseError::config_error(
            format!("Configuration file not found: {}", path.display()),
            Some(
                "Create a mockforge.json file or run 'mockforge init' to generate a default configuration",
            ),
        ));
    }

    // Read and parse configuration
    let content = fs::read_to_string(path).map_err(|e| {
        PulseError::config_error(
            format!("Cannot read configuration file: {}", e),
            Some("Check file permissions and ensure the file is readable"),
        )
    })?;

    let mut config: PulseConfig = serde_json::from_str(&content).map_err(|e| {
        PulseError::config_error(
            format!("Invalid JSON in configuration file: {}", e),
            Some("Check JSON syntax and ensure all required fields are present"),
        )
    })?;

    // Convert relative paths to absolute
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    resolve_relative_paths(&mut config, base_dir);

    // Validate configuration
    if let Err(validation_errors) = config.validate() {
        let error_message =
            crate::errors::ErrorFormatter::format_validation_errors(&validation_errors);
        return Err(PulseError::config_error(
            format!("Configuration validation failed:\n{}", error_message),
            Some("Fix the validation errors listed above"),
        ));
    }

    Ok(config)
}
/// Initialize a new configuration file with default values
pub fn init_config(path: &Path) -> PulseResult<()> {
    if path.exists() {
        return Err(PulseError::config_error(
            format!("Configuration file already exists: {}", path.display()),
            Some("Use --force to overwrite existing configuration or choose a different path"),
        ));
    }

    let default_config = generate_default_config();
    save_config(&default_config, path)?;

    println!("✅ Created default configuration at: {}", path.display());
    println!("💡 Edit the configuration file to customize settings for your project");

    Ok(())
}
/// Resolve relative paths in configuration to absolute paths
fn resolve_relative_paths(config: &mut PulseConfig, base_dir: &Path) {
    if !config.routes_dir.is_absolute() {
        config.routes_dir = base_dir.join(&config.routes_dir);
    }
    if !config.specs_dir.is_absolute() {
        config.specs_dir = base_dir.join(&config.specs_dir);
    }
    if !config.index_cache_path.is_absolute() {
        config.index_cache_path = base_dir.join(&config.index_cache_path);
    }
}

/// Generate a default configuration
pub fn generate_default_config() -> PulseConfig {
    PulseConfig {
        cypress_config_path: "cypress.config.ts".to_string(),
        base_url: "http://localhost:5173".to_string(),
        specs_pattern: "app/routes/**/test/*.cy.ts".to_string(),
        routes_dir: PathBuf::from("app/routes"),
        specs_dir: PathBuf::from("app/routes"),
        reports_dir: "cypress/reports".to_string(),
        index_cache_path: PathBuf::from(".mockforge/route-index.json"),
        default_timeout: 30000,
        server: ServerConfig::default(),
        execution: ExecutionConfig::default(),
        npm: NpmConfig::default(),
        ai: None,
        simulator: Some(crate::simulator::config::SimulatorConfig::default_config()),
        testcase: None,
        metrics: None,
    }
}

/// Save configuration to file
pub fn save_config(config: &PulseConfig, path: &Path) -> PulseResult<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot create config directory: {}", e),
                Some("Check permissions for the parent directory"),
            )
        })?;
    }

    // Serialize and write configuration
    let content = serde_json::to_string_pretty(config).map_err(|e| {
        PulseError::config_error(
            format!("Cannot serialize configuration: {}", e),
            None::<String>,
        )
    })?;

    fs::write(path, content).map_err(|e| {
        PulseError::fs_error(
            format!("Cannot write configuration file: {}", e),
            Some("Check write permissions for the target directory"),
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_validation() {
        let config = generate_default_config();
        // Default config should be valid (though some paths might not exist)
        // We'll test the structure is correct
        assert_eq!(config.base_url, "http://localhost:5173");
        assert_eq!(config.server.start_command, "npm run dev");
        assert_eq!(config.execution.mode, ExecutionMode::Development);
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let mut config = generate_default_config();
        config.base_url = "invalid-url".to_string();

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "base_url"));
    }

    #[test]
    fn test_config_validation_invalid_timeout() {
        let mut config = generate_default_config();
        config.default_timeout = 500; // Too low

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "default_timeout"));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let result = load_config(Path::new("nonexistent.json"));
        assert!(result.is_err());

        if let Err(PulseError::Configuration {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("not found"));
            assert!(suggestion.is_some());
        } else {
            panic!("Expected configuration error");
        }
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let mut original_config = generate_default_config();
        // Use temp dir paths to avoid validation errors
        original_config.routes_dir = temp_dir.path().join("routes");
        original_config.specs_dir = temp_dir.path().join("specs");
        original_config.index_cache_path = temp_dir.path().join("cache").join("index.json");
        if let Some(ref mut sim_cfg) = original_config.simulator {
            sim_cfg.services_dir = temp_dir.path().join("services");
        }

        // Create required directories
        fs::create_dir_all(&original_config.routes_dir).unwrap();
        fs::create_dir_all(&original_config.specs_dir).unwrap();
        fs::create_dir_all(original_config.index_cache_path.parent().unwrap()).unwrap();
        if let Some(ref sim_cfg) = original_config.simulator {
            fs::create_dir_all(&sim_cfg.services_dir).unwrap();
        }

        // Save config
        save_config(&original_config, &config_path).unwrap();

        // Load config back
        let loaded_config = load_config(&config_path).unwrap();

        assert_eq!(original_config.base_url, loaded_config.base_url);
        assert_eq!(original_config.specs_pattern, loaded_config.specs_pattern);
    }
}
