//! Configuration structures and loading for the API simulator

use crate::errors::ValidationError;
use crate::errors::{PulseError, PulseResult};
use crate::validation::{ConfigValidator, ValidationUtils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main simulator configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulatorConfig {
    /// Whether the simulator is enabled
    pub enabled: bool,
    /// Directory containing service definition YAML files
    pub services_dir: PathBuf,
    /// Port range for automatic port assignment
    pub port_range: PortRange,
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
            global_behavior: None,
        }
    }

    /// Create a default simulator configuration
    pub fn default_config() -> Self {
        Self {
            enabled: Self::check_environment_override(false),
            services_dir: PathBuf::from("services"),
            port_range: PortRange {
                start: 8000,
                end: 8999,
            },
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
    pub endpoints: Vec<EndpointDefinition>,
    #[serde(default)]
    pub behavior: Option<BehaviorConfig>,
}

/// Server configuration for a service
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub base_path: String,
    #[serde(default)]
    pub cors: Option<CorsConfig>,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    #[serde(default)]
    pub methods: Option<Vec<String>>,
    #[serde(default)]
    pub headers: Option<Vec<String>>,
}

/// Endpoint definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EndpointDefinition {
    pub method: String,
    pub path: String,
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Option<Vec<ParameterDefinition>>,
    #[serde(default)]
    pub request_body: Option<RequestBodyDefinition>,
    pub responses: HashMap<u16, ResponseDefinition>,
}

/// Parameter definition for endpoints
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParameterDefinition {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub description: Option<String>,
}

/// Parameter location (path, query, header)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
}

/// Request body definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestBodyDefinition {
    pub required: bool,
    pub schema: Option<String>, // Reference to model name
    pub content_type: Option<String>,
}

/// Response definition with templating support
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseDefinition {
    pub condition: Option<String>, // Template condition for conditional responses
    pub content_type: String,
    pub body: String, // Template string
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub side_effects: Option<Vec<SideEffect>>,
}

/// Side effects that can be triggered by responses
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SideEffect {
    pub action: String,
    pub target: String,
    pub value: String, // Template string
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

/// Configuration loader for service definitions
pub struct ConfigLoader {
    services_dir: PathBuf,
}

/// Result of loading service definitions with detailed statistics
#[derive(Debug, Clone)]
pub struct LoadResult {
    pub services: Vec<ServiceDefinition>,
    pub errors: Vec<LoadError>,
    pub files_scanned: usize,
    pub directories_scanned: usize,
}

/// Detailed error information for service loading
#[derive(Debug, Clone)]
pub struct LoadError {
    pub file_path: PathBuf,
    pub error_type: LoadErrorType,
    pub message: String,
}

/// Types of errors that can occur during service loading
#[derive(Debug, Clone, PartialEq)]
pub enum LoadErrorType {
    FileAccess,
    Parsing,
    Validation,
    DuplicateName,
    DirectoryAccess,
}

/// Summary of validation results
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub valid_count: usize,
    pub invalid_count: usize,
    pub total_files: usize,
    pub errors: Vec<LoadError>,
}

impl ValidationSummary {
    /// Check if all files are valid
    pub fn is_all_valid(&self) -> bool {
        self.invalid_count == 0 && self.valid_count > 0
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.valid_count as f64 / self.total_files as f64) * 100.0
        }
    }
}

impl LoadResult {
    /// Check if loading was successful (has services and no critical errors)
    pub fn is_successful(&self) -> bool {
        !self.services.is_empty()
    }

    /// Get services by name for quick lookup
    pub fn get_service_by_name(&self, name: &str) -> Option<&ServiceDefinition> {
        self.services.iter().find(|s| s.name == name)
    }

    /// Get all service names
    pub fn service_names(&self) -> Vec<&str> {
        self.services.iter().map(|s| s.name.as_str()).collect()
    }

    /// Get errors by type
    pub fn errors_by_type(&self, error_type: LoadErrorType) -> Vec<&LoadError> {
        self.errors
            .iter()
            .filter(|e| e.error_type == error_type)
            .collect()
    }
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new(services_dir: PathBuf) -> Self {
        Self { services_dir }
    }

    /// Load all service definitions from the services directory (recursively)
    pub fn load_all_services(&self) -> PulseResult<Vec<ServiceDefinition>> {
        if !self.services_dir.exists() {
            return Err(PulseError::config_error(
                format!(
                    "Services directory does not exist: {}",
                    self.services_dir.display()
                ),
                Some("Create the services directory and add YAML service definition files"),
            ));
        }

        let mut services = Vec::new();
        let mut load_errors = Vec::new();

        // Recursively scan for YAML files
        match self.scan_directory_recursive(&self.services_dir, &mut services, &mut load_errors) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }

        // Report load errors as warnings but continue
        for error in &load_errors {
            eprintln!("Warning: {}", error);
        }

        if services.is_empty() {
            let suggestion = if load_errors.is_empty() {
                "Add YAML files with service definitions to the services directory"
            } else {
                "Fix the validation errors in existing YAML files or add new valid service definitions"
            };

            return Err(PulseError::config_error(
                "No valid service definitions found in services directory",
                Some(suggestion),
            ));
        }

        Ok(services)
    }

    /// Recursively scan a directory for YAML service definition files
    fn scan_directory_recursive(
        &self,
        dir: &Path,
        services: &mut Vec<ServiceDefinition>,
        load_errors: &mut Vec<String>,
    ) -> PulseResult<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read directory {}: {}", dir.display(), e),
                Some("Check directory permissions"),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PulseError::fs_error(
                    format!("Error reading directory entry in {}: {}", dir.display(), e),
                    None::<String>,
                )
            })?;

            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                if let Err(e) = self.scan_directory_recursive(&path, services, load_errors) {
                    load_errors.push(format!(
                        "Failed to scan directory {}: {}",
                        path.display(),
                        e
                    ));
                }
            } else if path.is_file() && self.is_yaml_file(&path) {
                // Try to load YAML service definition
                match self.load_service(&path) {
                    Ok(service) => {
                        // Check for duplicate service names
                        if services.iter().any(|s| s.name == service.name) {
                            load_errors.push(format!(
                                "Duplicate service name '{}' found in {}, skipping",
                                service.name,
                                path.display()
                            ));
                        } else {
                            services.push(service);
                        }
                    }
                    Err(e) => {
                        load_errors.push(format!(
                            "Failed to load service from {}: {}",
                            path.display(),
                            e
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a file has a YAML extension
    fn is_yaml_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
            .unwrap_or(false)
    }

    /// Get all YAML files in the services directory (for file watching)
    pub fn get_all_yaml_files(&self) -> PulseResult<Vec<PathBuf>> {
        let mut yaml_files = Vec::new();

        if !self.services_dir.exists() {
            return Ok(yaml_files);
        }

        self.collect_yaml_files_recursive(&self.services_dir, &mut yaml_files)?;
        Ok(yaml_files)
    }

    /// Recursively collect all YAML files
    fn collect_yaml_files_recursive(
        &self,
        dir: &Path,
        yaml_files: &mut Vec<PathBuf>,
    ) -> PulseResult<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read directory {}: {}", dir.display(), e),
                Some("Check directory permissions"),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PulseError::fs_error(
                    format!("Error reading directory entry in {}: {}", dir.display(), e),
                    None::<String>,
                )
            })?;

            let path = entry.path();

            if path.is_dir() {
                // Skip hidden and common build directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                        || dir_name == "dist"
                        || dir_name == "build"
                    {
                        continue;
                    }
                }

                self.collect_yaml_files_recursive(&path, yaml_files)?;
            } else if path.is_file() && self.is_yaml_file(&path) {
                yaml_files.push(path);
            }
        }

        Ok(())
    }

    /// Validate all service definitions without loading them into memory
    pub fn validate_all_services(&self) -> PulseResult<ValidationSummary> {
        let mut valid_count = 0;
        let mut invalid_count = 0;
        let mut validation_errors = Vec::new();
        let mut files_scanned = 0;

        if !self.services_dir.exists() {
            return Err(PulseError::config_error(
                format!(
                    "Services directory does not exist: {}",
                    self.services_dir.display()
                ),
                Some("Create the services directory and add YAML service definition files"),
            ));
        }

        self.validate_directory_recursive(
            &self.services_dir,
            &mut valid_count,
            &mut invalid_count,
            &mut validation_errors,
            &mut files_scanned,
        )?;

        Ok(ValidationSummary {
            valid_count,
            invalid_count,
            total_files: files_scanned,
            errors: validation_errors,
        })
    }

    /// Recursively validate all YAML files in a directory
    fn validate_directory_recursive(
        &self,
        dir: &Path,
        valid_count: &mut usize,
        invalid_count: &mut usize,
        validation_errors: &mut Vec<LoadError>,
        files_scanned: &mut usize,
    ) -> PulseResult<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read directory {}: {}", dir.display(), e),
                Some("Check directory permissions"),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PulseError::fs_error(
                    format!("Error reading directory entry in {}: {}", dir.display(), e),
                    None::<String>,
                )
            })?;

            let path = entry.path();

            if path.is_dir() {
                // Skip hidden and build directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                        || dir_name == "dist"
                        || dir_name == "build"
                    {
                        continue;
                    }
                }

                self.validate_directory_recursive(
                    &path,
                    valid_count,
                    invalid_count,
                    validation_errors,
                    files_scanned,
                )?;
            } else if path.is_file() && self.is_yaml_file(&path) {
                *files_scanned += 1;

                match self.load_service(&path) {
                    Ok(_) => {
                        *valid_count += 1;
                    }
                    Err(e) => {
                        *invalid_count += 1;
                        let error_type = match &e {
                            PulseError::Configuration { .. } => LoadErrorType::Validation,
                            PulseError::FileSystem { .. } => LoadErrorType::FileAccess,
                            _ => LoadErrorType::Parsing,
                        };

                        validation_errors.push(LoadError {
                            file_path: path.clone(),
                            error_type,
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single service definition from a YAML file
    pub fn load_service(&self, path: &Path) -> PulseResult<ServiceDefinition> {
        let content = fs::read_to_string(path).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read service file {}: {}", path.display(), e),
                Some("Check file permissions and ensure the file exists"),
            )
        })?;

        let service: ServiceDefinition = serde_yaml::from_str(&content).map_err(|e| {
            PulseError::config_error(
                format!("Invalid YAML in service file {}: {}", path.display(), e),
                Some("Check YAML syntax and ensure all required fields are present"),
            )
        })?;

        self.validate_service(&service)?;
        Ok(service)
    }

    /// Validate a service definition
    pub fn validate_service(&self, service: &ServiceDefinition) -> PulseResult<()> {
        if let Err(validation_errors) = service.validate() {
            let error_message =
                crate::errors::ErrorFormatter::format_validation_errors(&validation_errors);
            return Err(PulseError::config_error(
                format!(
                    "Service validation failed for '{}':\n{}",
                    service.name, error_message
                ),
                Some("Fix the validation errors listed above"),
            ));
        }
        Ok(())
    }

    /// Load all services with detailed statistics and error reporting
    pub fn load_all_services_with_stats(&self) -> PulseResult<LoadResult> {
        if !self.services_dir.exists() {
            return Err(PulseError::config_error(
                format!(
                    "Services directory does not exist: {}",
                    self.services_dir.display()
                ),
                Some("Create the services directory and add YAML service definition files"),
            ));
        }

        let mut services = Vec::new();
        let mut load_errors = Vec::new();
        let mut files_scanned = 0;
        let mut directories_scanned = 0;

        match self.scan_directory_with_stats(
            &self.services_dir,
            &mut services,
            &mut load_errors,
            &mut files_scanned,
            &mut directories_scanned,
        ) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }

        let result = LoadResult {
            services,
            errors: load_errors,
            files_scanned,
            directories_scanned,
        };

        if result.services.is_empty() && !result.errors.is_empty() {
            return Err(PulseError::config_error(
                format!("No valid service definitions found. {} errors encountered while scanning {} files in {} directories", 
                    result.errors.len(), result.files_scanned, result.directories_scanned),
                Some("Fix the validation errors in existing YAML files or add new valid service definitions")
            ));
        } else if result.services.is_empty() {
            return Err(PulseError::config_error(
                format!(
                    "No service definitions found after scanning {} files in {} directories",
                    result.files_scanned, result.directories_scanned
                ),
                Some("Add YAML files with service definitions to the services directory"),
            ));
        }

        Ok(result)
    }

    /// Recursively scan directory with detailed statistics
    fn scan_directory_with_stats(
        &self,
        dir: &Path,
        services: &mut Vec<ServiceDefinition>,
        load_errors: &mut Vec<LoadError>,
        files_scanned: &mut usize,
        directories_scanned: &mut usize,
    ) -> PulseResult<()> {
        *directories_scanned += 1;

        let entries = fs::read_dir(dir).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read directory {}: {}", dir.display(), e),
                Some("Check directory permissions"),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                PulseError::fs_error(
                    format!("Error reading directory entry in {}: {}", dir.display(), e),
                    None::<String>,
                )
            })?;

            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common non-service directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                        || dir_name == "dist"
                        || dir_name == "build"
                    {
                        continue;
                    }
                }

                // Recursively scan subdirectories
                if let Err(e) = self.scan_directory_with_stats(
                    &path,
                    services,
                    load_errors,
                    files_scanned,
                    directories_scanned,
                ) {
                    load_errors.push(LoadError {
                        file_path: path.clone(),
                        error_type: LoadErrorType::DirectoryAccess,
                        message: format!("Failed to scan directory: {}", e),
                    });
                }
            } else if path.is_file() && self.is_yaml_file(&path) {
                *files_scanned += 1;

                // Try to load YAML service definition
                match self.load_service(&path) {
                    Ok(service) => {
                        // Check for duplicate service names
                        if services.iter().any(|s| s.name == service.name) {
                            load_errors.push(LoadError {
                                file_path: path.clone(),
                                error_type: LoadErrorType::DuplicateName,
                                message: format!(
                                    "Duplicate service name '{}' (already defined in another file)",
                                    service.name
                                ),
                            });
                        } else {
                            services.push(service);
                        }
                    }
                    Err(e) => {
                        let error_type = match &e {
                            PulseError::Configuration { .. } => LoadErrorType::Validation,
                            PulseError::FileSystem { .. } => LoadErrorType::FileAccess,
                            _ => LoadErrorType::Parsing,
                        };

                        load_errors.push(LoadError {
                            file_path: path.clone(),
                            error_type,
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

// Validation implementations
impl ConfigValidator for SimulatorConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate services directory
        if let Err(e) =
            ValidationUtils::validate_directory(&self.services_dir, "services_dir", false)
        {
            errors.push(e);
        }

        // Validate port range
        if self.port_range.start >= self.port_range.end {
            errors.push(ValidationError {
                field: "port_range".to_string(),
                message: "Port range start must be less than end".to_string(),
                suggestion: Some("Ensure start port is less than end port".to_string()),
            });
        }

        if self.port_range.start < 1024 {
            errors.push(ValidationError {
                field: "port_range.start".to_string(),
                message: "Port range start should be >= 1024 to avoid system ports".to_string(),
                suggestion: Some("Use ports 1024 or higher".to_string()),
            });
        }

        // Note: u16 max value is 65535, so this check is not needed
        // but we keep it for clarity and future-proofing

        // Validate global behavior if present
        if let Some(ref behavior) = self.global_behavior {
            if let Err(mut behavior_errors) = behavior.validate() {
                errors.append(&mut behavior_errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for ServiceDefinition {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate service name
        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.name, "name") {
            errors.push(e);
        }

        // Validate server config
        if let Err(mut server_errors) = self.server.validate() {
            errors.append(&mut server_errors);
        }

        // Validate endpoints
        if self.endpoints.is_empty() {
            errors.push(ValidationError {
                field: "endpoints".to_string(),
                message: "Service must have at least one endpoint".to_string(),
                suggestion: Some("Add at least one endpoint definition".to_string()),
            });
        }

        for (i, endpoint) in self.endpoints.iter().enumerate() {
            if let Err(mut endpoint_errors) = endpoint.validate() {
                // Prefix field names with endpoint index
                for error in &mut endpoint_errors {
                    error.field = format!("endpoints[{}].{}", i, error.field);
                }
                errors.append(&mut endpoint_errors);
            }
        }

        // Validate model schemas if present
        if let Some(ref models) = self.models {
            if let Err(mut model_errors) = self.validate_models(models) {
                errors.append(&mut model_errors);
            }
        }

        // Validate fixtures against models if both are present
        if let Some(ref models) = self.models {
            if let Some(ref fixtures) = self.fixtures {
                if let Err(mut fixture_errors) =
                    self.validate_fixtures_against_models(fixtures, models)
                {
                    errors.append(&mut fixture_errors);
                }
            }
        }

        // Validate endpoint schema references
        if let Err(mut schema_ref_errors) = self.validate_schema_references() {
            errors.append(&mut schema_ref_errors);
        }

        // Validate behavior if present
        if let Some(ref behavior) = self.behavior {
            if let Err(mut behavior_errors) = behavior.validate() {
                errors.append(&mut behavior_errors);
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

        // Validate base path
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.base_path, "server.base_path")
        {
            errors.push(e);
        }

        if !self.base_path.starts_with('/') {
            errors.push(ValidationError {
                field: "server.base_path".to_string(),
                message: "Base path must start with '/'".to_string(),
                suggestion: Some("Ensure base path starts with '/', e.g., '/api/v1'".to_string()),
            });
        }

        // Validate port if specified
        if let Some(port) = self.port {
            if port < 1024 {
                errors.push(ValidationError {
                    field: "server.port".to_string(),
                    message: "Port should be >= 1024 to avoid system ports".to_string(),
                    suggestion: Some("Use ports 1024 or higher".to_string()),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for EndpointDefinition {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate HTTP method
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&self.method.to_uppercase().as_str()) {
            errors.push(ValidationError {
                field: "method".to_string(),
                message: format!(
                    "Invalid HTTP method '{}'. Must be one of: {}",
                    self.method,
                    valid_methods.join(", ")
                ),
                suggestion: Some(
                    "Use a valid HTTP method like GET, POST, PUT, DELETE, etc.".to_string(),
                ),
            });
        }

        // Validate path
        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.path, "path") {
            errors.push(e);
        }

        if !self.path.starts_with('/') {
            errors.push(ValidationError {
                field: "path".to_string(),
                message: "Endpoint path must start with '/'".to_string(),
                suggestion: Some("Ensure path starts with '/', e.g., '/users'".to_string()),
            });
        }

        // Validate responses
        if self.responses.is_empty() {
            errors.push(ValidationError {
                field: "responses".to_string(),
                message: "Endpoint must have at least one response definition".to_string(),
                suggestion: Some("Add at least one response (e.g., 200 status)".to_string()),
            });
        }

        for (status_code, response) in &self.responses {
            if *status_code < 100 || *status_code > 599 {
                errors.push(ValidationError {
                    field: format!("responses.{}", status_code),
                    message: "HTTP status code must be between 100 and 599".to_string(),
                    suggestion: Some("Use valid HTTP status codes (100-599)".to_string()),
                });
            }

            if let Err(mut response_errors) = response.validate_with_status_code(*status_code) {
                for error in &mut response_errors {
                    error.field = format!("responses.{}.{}", status_code, error.field);
                }
                errors.append(&mut response_errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for ResponseDefinition {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        // Use default validation without status code context
        self.validate_with_status_code(200)
    }
}

impl ResponseDefinition {
    /// Validate response definition with status code context
    pub fn validate_with_status_code(&self, status_code: u16) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate content type
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.content_type, "content_type")
        {
            errors.push(e);
        }

        // Validate body template based on status code
        let body_trimmed = self.body.trim();
        if body_trimmed.is_empty() {
            // Empty body is allowed for certain HTTP status codes
            match status_code {
                204 | 304 => {
                    // 204 No Content and 304 Not Modified should have empty bodies
                }
                _ => {
                    // For other status codes, warn but don't fail validation
                    // as empty bodies might be intentional in some cases
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

impl ConfigValidator for BehaviorConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate latency config if present
        if let Some(ref latency) = self.latency {
            if latency.min_ms > latency.max_ms {
                errors.push(ValidationError {
                    field: "behavior.latency".to_string(),
                    message: "Minimum latency must be less than or equal to maximum latency"
                        .to_string(),
                    suggestion: Some("Ensure min_ms <= max_ms".to_string()),
                });
            }
        }

        // Validate error simulation config if present
        if let Some(ref error_sim) = self.error_simulation {
            if error_sim.rate < 0.0 || error_sim.rate > 1.0 {
                errors.push(ValidationError {
                    field: "behavior.error_simulation.rate".to_string(),
                    message: "Error rate must be between 0.0 and 1.0".to_string(),
                    suggestion: Some(
                        "Use a decimal value between 0.0 and 1.0 (e.g., 0.05 for 5%)".to_string(),
                    ),
                });
            }
        }

        // Validate rate limiting config if present
        if let Some(ref rate_limit) = self.rate_limiting {
            if rate_limit.requests_per_minute == 0 {
                errors.push(ValidationError {
                    field: "behavior.rate_limiting.requests_per_minute".to_string(),
                    message: "Requests per minute must be greater than 0".to_string(),
                    suggestion: Some("Set a positive number of requests per minute".to_string()),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ServiceDefinition {
    /// Validate model schemas for basic JSON Schema compliance
    fn validate_models(
        &self,
        models: &HashMap<String, serde_json::Value>,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (model_name, schema) in models {
            // Validate model name
            if let Err(e) = ValidationUtils::validate_non_empty_string(
                model_name,
                &format!("models.{}", model_name),
            ) {
                errors.push(e);
            }

            // Basic JSON Schema validation - check for required fields
            if let Some(schema_obj) = schema.as_object() {
                // Check for type field
                if !schema_obj.contains_key("type") {
                    errors.push(
                        ValidationError::new(
                            format!("models.{}.type", model_name),
                            "Model schema must have a 'type' field",
                        )
                        .with_suggestion("Add a 'type' field with value 'object' for most models"),
                    );
                }

                // If type is object, validate properties
                if let Some(type_val) = schema_obj.get("type") {
                    if type_val == "object" {
                        if !schema_obj.contains_key("properties") {
                            errors.push(
                                ValidationError::new(
                                    format!("models.{}.properties", model_name),
                                    "Object type models should have a 'properties' field",
                                )
                                .with_suggestion(
                                    "Add a 'properties' field defining the object's properties",
                                ),
                            );
                        }
                    }
                }
            } else {
                errors.push(
                    ValidationError::new(
                        format!("models.{}", model_name),
                        "Model schema must be a JSON object",
                    )
                    .with_suggestion("Define the model as a JSON object with type and properties"),
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate fixtures against model schemas for basic type compliance
    fn validate_fixtures_against_models(
        &self,
        fixtures: &HashMap<String, serde_json::Value>,
        models: &HashMap<String, serde_json::Value>,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (fixture_name, fixture_data) in fixtures {
            // Check if there's a corresponding model (optional validation)
            if let Some(model_schema) = models.get(fixture_name) {
                if let Some(fixture_array) = fixture_data.as_array() {
                    for (i, item) in fixture_array.iter().enumerate() {
                        if let Err(mut item_errors) = self.validate_fixture_item_basic(
                            item,
                            model_schema,
                            &format!("fixtures.{}[{}]", fixture_name, i),
                        ) {
                            errors.append(&mut item_errors);
                        }
                    }
                } else if let Err(mut item_errors) = self.validate_fixture_item_basic(
                    fixture_data,
                    model_schema,
                    &format!("fixtures.{}", fixture_name),
                ) {
                    errors.append(&mut item_errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Basic validation of a fixture item against a model schema
    fn validate_fixture_item_basic(
        &self,
        item: &serde_json::Value,
        schema: &serde_json::Value,
        field_path: &str,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Some(schema_obj) = schema.as_object() {
            if let Some(properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
                if let Some(item_obj) = item.as_object() {
                    // Check for required properties (basic check)
                    if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
                        for req_field in required {
                            if let Some(field_name) = req_field.as_str() {
                                if !item_obj.contains_key(field_name) {
                                    errors.push(
                                        ValidationError::new(
                                            format!("{}.{}", field_path, field_name),
                                            format!("Required field '{}' is missing", field_name),
                                        )
                                        .with_suggestion(
                                            format!(
                                                "Add the required field '{}' to the fixture item",
                                                field_name
                                            ),
                                        ),
                                    );
                                }
                            }
                        }
                    }

                    // Basic type checking for existing properties
                    for (prop_name, prop_value) in item_obj {
                        if let Some(prop_schema) = properties.get(prop_name) {
                            if let Some(expected_type) =
                                prop_schema.get("type").and_then(|t| t.as_str())
                            {
                                let actual_type = match prop_value {
                                    serde_json::Value::String(_) => "string",
                                    serde_json::Value::Number(_) => "number",
                                    serde_json::Value::Bool(_) => "boolean",
                                    serde_json::Value::Array(_) => "array",
                                    serde_json::Value::Object(_) => "object",
                                    serde_json::Value::Null => "null",
                                };

                                if expected_type != actual_type
                                    && !(expected_type == "integer" && actual_type == "number")
                                {
                                    errors.push(
                                        ValidationError::new(
                                            format!("{}.{}", field_path, prop_name),
                                            format!(
                                                "Expected type '{}' but found '{}'",
                                                expected_type, actual_type
                                            ),
                                        )
                                        .with_suggestion(
                                            format!(
                                                "Change the value to match the expected type '{}'",
                                                expected_type
                                            ),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                } else {
                    errors.push(
                        ValidationError::new(
                            field_path.to_string(),
                            "Expected object type for fixture item",
                        )
                        .with_suggestion("Ensure fixture items are JSON objects"),
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

    /// Validate that schema references in endpoints point to existing models
    fn validate_schema_references(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (i, endpoint) in self.endpoints.iter().enumerate() {
            // Check request body schema references
            if let Some(ref request_body) = endpoint.request_body {
                if let Some(ref schema_ref) = request_body.schema {
                    if let Some(ref models) = self.models {
                        if !models.contains_key(schema_ref) {
                            errors.push(ValidationError::new(
                                format!("endpoints[{}].request_body.schema", i),
                                format!("Schema reference '{}' not found in models", schema_ref)
                            ).with_suggestion(format!("Define the '{}' model in the models section or use an existing model name", schema_ref)));
                        }
                    } else {
                        errors.push(ValidationError::new(
                            format!("endpoints[{}].request_body.schema", i),
                            format!("Schema reference '{}' used but no models are defined", schema_ref)
                        ).with_suggestion("Define models in the models section or remove the schema reference"));
                    }
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

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            services_dir: PathBuf::from("services"),
            port_range: PortRange {
                start: 8000,
                end: 8999,
            },
            global_behavior: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_simulator_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");
        fs::create_dir_all(&services_dir).unwrap();

        let config = SimulatorConfig {
            enabled: true,
            services_dir,
            port_range: PortRange {
                start: 8000,
                end: 8999,
            },
            global_behavior: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port_range() {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");
        fs::create_dir_all(&services_dir).unwrap();

        let config = SimulatorConfig {
            enabled: true,
            services_dir,
            port_range: PortRange {
                start: 9000,
                end: 8000, // Invalid: start > end
            },
            global_behavior: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "port_range"));
    }

    #[test]
    fn test_service_definition_validation() {
        let service = ServiceDefinition {
            name: "test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service".to_string()),
            server: ServerConfig {
                port: Some(8001),
                base_path: "/api/v1".to_string(),
                cors: None,
            },
            models: None,
            fixtures: None,
            endpoints: vec![EndpointDefinition {
                method: "GET".to_string(),
                path: "/test".to_string(),
                description: None,
                parameters: None,
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"message": "test"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        },
                    );
                    responses
                },
            }],
            behavior: None,
        };

        assert!(service.validate().is_ok());
    }

    #[test]
    fn test_yaml_loading() {
        let yaml_content = r#"
name: 'test-service'
version: '1.0.0'
description: 'Test service'
server:
  port: 8001
  base_path: '/api/v1'
fixtures:
  users:
    - id: 1
      name: 'Alice'
endpoints:
  - method: GET
    path: '/users'
    responses:
      200:
        content_type: 'application/json'
        body: '{{ fixtures.users }}'
"#;

        let service: ServiceDefinition = serde_yaml::from_str(yaml_content).unwrap();
        assert_eq!(service.name, "test-service");
        assert_eq!(service.version, Some("1.0.0".to_string()));
        assert_eq!(service.server.base_path, "/api/v1");
        assert_eq!(service.endpoints.len(), 1);
        assert_eq!(service.endpoints[0].method, "GET");
        assert_eq!(service.endpoints[0].path, "/users");

        // Validate the loaded service
        assert!(service.validate().is_ok());
    }

    #[test]
    fn test_invalid_endpoint_method() {
        let service = ServiceDefinition {
            name: "test-service".to_string(),
            version: None,
            description: None,
            server: ServerConfig {
                port: None,
                base_path: "/api".to_string(),
                cors: None,
            },
            models: None,
            fixtures: None,
            endpoints: vec![EndpointDefinition {
                method: "INVALID".to_string(), // Invalid HTTP method
                path: "/test".to_string(),
                description: None,
                parameters: None,
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"message": "test"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        },
                    );
                    responses
                },
            }],
            behavior: None,
        };

        let result = service.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field.contains("method")));
    }

    #[test]
    fn test_model_validation() {
        let mut models = HashMap::new();

        // Valid model
        models.insert(
            "User".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer"},
                    "name": {"type": "string"}
                },
                "required": ["id", "name"]
            }),
        );

        // Invalid model (missing type)
        models.insert(
            "InvalidModel".to_string(),
            serde_json::json!({
                "properties": {
                    "field": {"type": "string"}
                }
            }),
        );

        let service = ServiceDefinition {
            name: "test-service".to_string(),
            version: None,
            description: None,
            server: ServerConfig {
                port: None,
                base_path: "/api".to_string(),
                cors: None,
            },
            models: Some(models),
            fixtures: None,
            endpoints: vec![EndpointDefinition {
                method: "GET".to_string(),
                path: "/test".to_string(),
                description: None,
                parameters: None,
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"message": "test"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        },
                    );
                    responses
                },
            }],
            behavior: None,
        };

        let result = service.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field.contains("InvalidModel.type")));
    }

    #[test]
    fn test_fixture_validation_against_models() {
        let mut models = HashMap::new();
        models.insert(
            "User".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "integer"},
                    "name": {"type": "string"},
                    "email": {"type": "string"}
                },
                "required": ["id", "name"]
            }),
        );

        let mut fixtures = HashMap::new();
        fixtures.insert(
            "User".to_string(),
            serde_json::json!([
                {
                    "id": 1,
                    "name": "Alice",
                    "email": "alice@example.com"
                },
                {
                    "id": "invalid", // Should be integer
                    "name": "Bob"
                    // Missing required email is OK for basic validation
                }
            ]),
        );

        let service = ServiceDefinition {
            name: "test-service".to_string(),
            version: None,
            description: None,
            server: ServerConfig {
                port: None,
                base_path: "/api".to_string(),
                cors: None,
            },
            models: Some(models),
            fixtures: Some(fixtures),
            endpoints: vec![EndpointDefinition {
                method: "GET".to_string(),
                path: "/test".to_string(),
                description: None,
                parameters: None,
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"message": "test"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        },
                    );
                    responses
                },
            }],
            behavior: None,
        };

        let result = service.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field.contains("fixtures.User[1].id")));
    }

    #[test]
    fn test_schema_reference_validation() {
        let service = ServiceDefinition {
            name: "test-service".to_string(),
            version: None,
            description: None,
            server: ServerConfig {
                port: None,
                base_path: "/api".to_string(),
                cors: None,
            },
            models: None, // No models defined
            fixtures: None,
            endpoints: vec![EndpointDefinition {
                method: "POST".to_string(),
                path: "/test".to_string(),
                description: None,
                parameters: None,
                request_body: Some(RequestBodyDefinition {
                    required: true,
                    schema: Some("NonExistentModel".to_string()), // Reference to non-existent model
                    content_type: Some("application/json".to_string()),
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"message": "test"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        },
                    );
                    responses
                },
            }],
            behavior: None,
        };

        let result = service.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.field.contains("request_body.schema")));
    }

    #[test]
    fn test_config_loader_recursive_scanning() {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");

        // Create nested directory structure
        let subdir = services_dir.join("v1").join("users");
        fs::create_dir_all(&subdir).unwrap();

        // Create valid service file in root
        let service1_path = services_dir.join("service1.yaml");
        fs::write(
            &service1_path,
            r#"
name: 'service1'
server:
  base_path: '/api/v1/service1'
endpoints:
  - method: GET
    path: '/test'
    responses:
      200:
        content_type: 'application/json'
        body: '{"message": "test"}'
"#,
        )
        .unwrap();

        // Create valid service file in subdirectory
        let service2_path = subdir.join("users.yml");
        fs::write(
            &service2_path,
            r#"
name: 'users-service'
server:
  base_path: '/api/v1/users'
endpoints:
  - method: GET
    path: '/users'
    responses:
      200:
        content_type: 'application/json'
        body: '{"users": []}'
"#,
        )
        .unwrap();

        // Create invalid service file
        let invalid_path = services_dir.join("invalid.yaml");
        fs::write(
            &invalid_path,
            r#"
name: 'invalid-service'
# Missing server config and endpoints
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(services_dir);
        let result = loader.load_all_services_with_stats().unwrap();

        assert_eq!(result.services.len(), 2);
        assert_eq!(result.files_scanned, 3);
        assert!(result.directories_scanned >= 3); // root + v1 + users
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, LoadErrorType::Validation);

        // Check that both services were loaded
        let service_names: Vec<&str> = result.service_names();
        assert!(service_names.contains(&"service1"));
        assert!(service_names.contains(&"users-service"));
    }

    #[test]
    fn test_config_loader_duplicate_names() {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");
        fs::create_dir_all(&services_dir).unwrap();

        // Create two services with the same name
        let service1_path = services_dir.join("service1.yaml");
        fs::write(
            &service1_path,
            r#"
name: 'duplicate-service'
server:
  base_path: '/api/v1/service1'
endpoints:
  - method: GET
    path: '/test1'
    responses:
      200:
        content_type: 'application/json'
        body: '{"message": "test1"}'
"#,
        )
        .unwrap();

        let service2_path = services_dir.join("service2.yaml");
        fs::write(
            &service2_path,
            r#"
name: 'duplicate-service'
server:
  base_path: '/api/v1/service2'
endpoints:
  - method: GET
    path: '/test2'
    responses:
      200:
        content_type: 'application/json'
        body: '{"message": "test2"}'
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(services_dir);
        let result = loader.load_all_services_with_stats().unwrap();

        // Should load only one service and report duplicate error
        assert_eq!(result.services.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, LoadErrorType::DuplicateName);
        assert!(result.errors[0].message.contains("Duplicate service name"));
    }

    #[test]
    fn test_validation_summary() {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");
        fs::create_dir_all(&services_dir).unwrap();

        // Create one valid and one invalid service
        let valid_path = services_dir.join("valid.yaml");
        fs::write(
            &valid_path,
            r#"
name: 'valid-service'
server:
  base_path: '/api/valid'
endpoints:
  - method: GET
    path: '/test'
    responses:
      200:
        content_type: 'application/json'
        body: '{"message": "valid"}'
"#,
        )
        .unwrap();

        let invalid_path = services_dir.join("invalid.yaml");
        fs::write(
            &invalid_path,
            r#"
name: 'invalid-service'
# Missing required fields
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(services_dir);
        let summary = loader.validate_all_services().unwrap();

        assert_eq!(summary.valid_count, 1);
        assert_eq!(summary.invalid_count, 1);
        assert_eq!(summary.total_files, 2);
        assert!(!summary.is_all_valid());
        assert_eq!(summary.success_rate(), 50.0);
        assert_eq!(summary.errors.len(), 1);
    }

    #[test]
    fn test_yaml_file_detection() {
        let loader = ConfigLoader::new(PathBuf::from("test"));

        assert!(loader.is_yaml_file(Path::new("service.yaml")));
        assert!(loader.is_yaml_file(Path::new("service.yml")));
        assert!(loader.is_yaml_file(Path::new("SERVICE.YAML")));
        assert!(loader.is_yaml_file(Path::new("service.YML")));
        assert!(!loader.is_yaml_file(Path::new("service.json")));
        assert!(!loader.is_yaml_file(Path::new("service.txt")));
        assert!(!loader.is_yaml_file(Path::new("service")));
    }

    #[test]
    fn test_load_result_utilities() {
        let services = vec![
            ServiceDefinition {
                name: "service1".to_string(),
                version: None,
                description: None,
                server: ServerConfig {
                    port: None,
                    base_path: "/api/service1".to_string(),
                    cors: None,
                },
                models: None,
                fixtures: None,
                endpoints: vec![],
                behavior: None,
            },
            ServiceDefinition {
                name: "service2".to_string(),
                version: None,
                description: None,
                server: ServerConfig {
                    port: None,
                    base_path: "/api/service2".to_string(),
                    cors: None,
                },
                models: None,
                fixtures: None,
                endpoints: vec![],
                behavior: None,
            },
        ];

        let errors = vec![
            LoadError {
                file_path: PathBuf::from("error1.yaml"),
                error_type: LoadErrorType::Validation,
                message: "Validation error".to_string(),
            },
            LoadError {
                file_path: PathBuf::from("error2.yaml"),
                error_type: LoadErrorType::Parsing,
                message: "Parsing error".to_string(),
            },
        ];

        let result = LoadResult {
            services,
            errors,
            files_scanned: 4,
            directories_scanned: 2,
        };

        assert!(result.is_successful());
        assert_eq!(result.service_names(), vec!["service1", "service2"]);
        assert!(result.get_service_by_name("service1").is_some());
        assert!(result.get_service_by_name("nonexistent").is_none());

        let validation_errors = result.errors_by_type(LoadErrorType::Validation);
        assert_eq!(validation_errors.len(), 1);
        assert_eq!(validation_errors[0].message, "Validation error");
    }

    #[test]
    fn test_example_service_loading() {
        // Test loading the example service file from the project
        let example_path = Path::new("../../services/example-user-service.yaml");

        if example_path.exists() {
            let loader = ConfigLoader::new(PathBuf::from("../../services"));

            // Test loading the specific example file
            match loader.load_service(example_path) {
                Ok(service) => {
                    assert_eq!(service.name, "user-service");
                    assert_eq!(service.version, Some("1.0.0".to_string()));
                    assert_eq!(service.server.base_path, "/api/v1/users");
                    assert_eq!(service.server.port, Some(8001));
                    assert!(service.models.is_some());
                    assert!(service.fixtures.is_some());
                    assert!(!service.endpoints.is_empty());
                    assert!(service.behavior.is_some());

                    // Validate the service
                    assert!(service.validate().is_ok());
                }
                Err(e) => {
                    panic!("Failed to load example service: {}", e);
                }
            }

            // Test loading all services from the services directory
            match loader.load_all_services_with_stats() {
                Ok(result) => {
                    assert!(result.is_successful());
                    assert!(result.get_service_by_name("user-service").is_some());
                }
                Err(e) => {
                    panic!("Failed to load services from directory: {}", e);
                }
            }
        } else {
            // Skip test if example file doesn't exist
            println!(
                "Skipping example service test - file not found at {:?}",
                example_path
            );
        }
    }

    #[test]
    fn test_environment_variable_override() {
        // Save original environment variable state
        let original_env = env::var("PULSE_API_SIMULATOR").ok();

        // Test that environment variable overrides configuration
        let config = SimulatorConfig {
            enabled: false,
            services_dir: PathBuf::from("services"),
            port_range: PortRange {
                start: 8000,
                end: 8999,
            },
            global_behavior: None,
        };

        // Without environment variable, should use config value
        env::remove_var("PULSE_API_SIMULATOR");
        assert!(!config.is_enabled());

        // With environment variable set to true, should override config
        env::set_var("PULSE_API_SIMULATOR", "true");
        assert!(config.is_enabled());

        // Test various true values
        for true_value in &["true", "TRUE", "1", "yes", "YES", "on", "ON"] {
            env::set_var("PULSE_API_SIMULATOR", true_value);
            assert!(config.is_enabled(), "Failed for value: {}", true_value);
        }

        // Test false values
        for false_value in &["false", "FALSE", "0", "no", "NO", "off", "OFF"] {
            env::set_var("PULSE_API_SIMULATOR", false_value);
            assert!(!config.is_enabled(), "Failed for value: {}", false_value);
        }

        // Restore original environment variable state
        match original_env {
            Some(value) => env::set_var("PULSE_API_SIMULATOR", value),
            None => env::remove_var("PULSE_API_SIMULATOR"),
        }
    }

    #[test]
    fn test_effective_enabled_state() {
        // Save original environment variable state
        let original_env = env::var("PULSE_API_SIMULATOR").ok();

        let config = SimulatorConfig {
            enabled: false,
            services_dir: PathBuf::from("services"),
            port_range: PortRange {
                start: 8000,
                end: 8999,
            },
            global_behavior: None,
        };

        // Without environment variable
        env::remove_var("PULSE_API_SIMULATOR");
        let (enabled, env_override) = config.effective_enabled_state();
        assert!(!enabled);
        assert!(!env_override);

        // With environment variable
        env::set_var("PULSE_API_SIMULATOR", "true");
        let (enabled, env_override) = config.effective_enabled_state();
        assert!(enabled);
        assert!(env_override);

        // Restore original environment variable state
        match original_env {
            Some(value) => env::set_var("PULSE_API_SIMULATOR", value),
            None => env::remove_var("PULSE_API_SIMULATOR"),
        }
    }

    #[test]
    fn test_default_config() {
        // Save original environment variable state
        let original_env = env::var("PULSE_API_SIMULATOR").ok();

        // Test that default config respects environment variables
        env::remove_var("PULSE_API_SIMULATOR");
        let config = SimulatorConfig::default_config();
        assert!(!config.is_enabled());

        env::set_var("PULSE_API_SIMULATOR", "true");
        let config = SimulatorConfig::default_config();
        assert!(config.is_enabled());

        // Restore original environment variable state
        match original_env {
            Some(value) => env::set_var("PULSE_API_SIMULATOR", value),
            None => env::remove_var("PULSE_API_SIMULATOR"),
        }
    }
}
