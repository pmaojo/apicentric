//! Defines the error types and error handling utilities for the Apicentric application.
//!
//! This module includes the main `ApicentricError` enum, which represents all possible
//! errors that can occur within the application. It provides structured error types
//! with contextual information and actionable suggestions for common issues.

use std::fmt;

/// The main error type for all Apicentric operations.
///
/// This enum consolidates all possible errors that can occur within the application,
/// providing contextual variants with built-in suggestions for common issues.
#[derive(Debug)]
pub enum ApicentricError {
    /// An error related to application configuration.
    Configuration {
        message: String,
        suggestion: Option<String>,
    },

    /// An error that occurred within the server.
    Server {
        message: String,
        suggestion: Option<String>,
    },

    /// An error that occurred during test execution.
    TestExecution {
        message: String,
        suggestion: Option<String>,
    },

    /// An error related to file system operations.
    FileSystem {
        message: String,
        suggestion: Option<String>,
    },

    /// An error that occurred during data validation.
    Validation {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },

    /// A general-purpose runtime error.
    Runtime {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to simulated services.
    Service {
        message: String,
        service_name: Option<String>,
        suggestion: Option<String>,
    },

    /// Errors related to AI operations.
    Ai {
        message: String,
        provider: Option<String>,
        suggestion: Option<String>,
    },

    /// Errors related to recording functionality.
    Recording {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to authentication and authorization.
    Authentication {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to network operations.
    Network {
        message: String,
        url: Option<String>,
        suggestion: Option<String>,
    },

    /// Errors related to database operations.
    Database {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to scripting (Rhai, etc).
    Scripting {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to MQTT operations.
    #[cfg(feature = "iot")]
    Mqtt {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to Modbus operations.
    #[cfg(feature = "iot")]
    Modbus {
        message: String,
        suggestion: Option<String>,
    },

    /// Errors related to CSV parsing.
    Csv {
        message: String,
        suggestion: Option<String>,
    },

    /// General data processing errors.
    Data {
        message: String,
        suggestion: Option<String>,
    },

    /// An I/O error.
    Io(std::io::Error),

    /// An error that occurred during JSON serialization or deserialization.
    Json(serde_json::Error),

    /// An error related to glob pattern matching.
    Glob(glob::GlobError),

    /// An error in a glob pattern.
    Pattern(glob::PatternError),
}

impl fmt::Display for ApicentricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration { message, .. } => write!(f, "Configuration error: {}", message),
            Self::Server { message, .. } => write!(f, "Server error: {}", message),
            Self::TestExecution { message, .. } => write!(f, "Test execution error: {}", message),
            Self::FileSystem { message, .. } => write!(f, "File system error: {}", message),
            Self::Validation { message, field, .. } => {
                if let Some(field) = field {
                    write!(f, "Validation error in field '{}': {}", field, message)
                } else {
                    write!(f, "Validation error: {}", message)
                }
            }
            Self::Runtime { message, .. } => write!(f, "Runtime error: {}", message),
            Self::Service {
                message,
                service_name,
                ..
            } => {
                if let Some(name) = service_name {
                    write!(f, "Service error for '{}': {}", name, message)
                } else {
                    write!(f, "Service error: {}", message)
                }
            }
            Self::Ai {
                message, provider, ..
            } => {
                if let Some(provider) = provider {
                    write!(f, "AI error ({}): {}", provider, message)
                } else {
                    write!(f, "AI error: {}", message)
                }
            }
            Self::Recording { message, .. } => write!(f, "Recording error: {}", message),
            Self::Authentication { message, .. } => write!(f, "Authentication error: {}", message),
            Self::Network { message, url, .. } => {
                if let Some(url) = url {
                    write!(f, "Network error for {}: {}", url, message)
                } else {
                    write!(f, "Network error: {}", message)
                }
            }
            Self::Database { message, .. } => write!(f, "Database error: {}", message),
            Self::Scripting { message, .. } => write!(f, "Scripting error: {}", message),
            #[cfg(feature = "iot")]
            Self::Mqtt { message, .. } => write!(f, "MQTT error: {}", message),
            #[cfg(feature = "iot")]
            Self::Modbus { message, .. } => write!(f, "Modbus error: {}", message),
            Self::Csv { message, .. } => write!(f, "CSV error: {}", message),
            Self::Data { message, .. } => write!(f, "Data error: {}", message),
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Json(err) => write!(f, "JSON parsing error: {}", err),
            Self::Glob(err) => write!(f, "Glob pattern error: {}", err),
            Self::Pattern(err) => write!(f, "Pattern error: {}", err),
        }
    }
}

impl std::error::Error for ApicentricError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::Glob(err) => Some(err),
            Self::Pattern(err) => Some(err),
            _ => None,
        }
    }
}

impl ApicentricError {
    /// Creates a new configuration error.
    pub fn config_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Configuration {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new validation error.
    pub fn validation_error(
        message: impl Into<String>,
        field: Option<impl Into<String>>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.map(|f| f.into()),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new file system error.
    pub fn fs_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::FileSystem {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new server error.
    pub fn server_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Server {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new test execution error.
    pub fn test_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::TestExecution {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new runtime error.
    pub fn runtime_error(
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Runtime {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new service error.
    pub fn service_error(
        message: impl Into<String>,
        service_name: Option<impl Into<String>>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Service {
            message: message.into(),
            service_name: service_name.map(|s| s.into()),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new AI error.
    pub fn ai_error(
        message: impl Into<String>,
        provider: Option<impl Into<String>>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Ai {
            message: message.into(),
            provider: provider.map(|p| p.into()),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new recording error.
    pub fn recording_error(
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Recording {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new authentication error.
    pub fn auth_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Authentication {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new network error.
    pub fn network_error(
        message: impl Into<String>,
        url: Option<impl Into<String>>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            url: url.map(|u| u.into()),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new database error.
    pub fn database_error(
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Database {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new scripting error.
    pub fn scripting_error(
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Scripting {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new data error.
    pub fn data_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Data {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Returns the suggestion for this error, if any.
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::Configuration { suggestion, .. }
            | Self::Server { suggestion, .. }
            | Self::TestExecution { suggestion, .. }
            | Self::FileSystem { suggestion, .. }
            | Self::Validation { suggestion, .. }
            | Self::Runtime { suggestion, .. }
            | Self::Service { suggestion, .. }
            | Self::Ai { suggestion, .. }
            | Self::Recording { suggestion, .. }
            | Self::Authentication { suggestion, .. }
            | Self::Network { suggestion, .. }
            | Self::Database { suggestion, .. }
            | Self::Scripting { suggestion, .. }
            | Self::Csv { suggestion, .. }
            | Self::Data { suggestion, .. } => suggestion.as_deref(),
            #[cfg(feature = "iot")]
            Self::Mqtt { suggestion, .. } | Self::Modbus { suggestion, .. } => {
                suggestion.as_deref()
            }
            _ => None,
        }
    }

    /// Returns the field name for validation errors, if any.
    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Validation { field, .. } => field.as_deref(),
            _ => None,
        }
    }

    /// Returns the service name for service errors, if any.
    pub fn service_name(&self) -> Option<&str> {
        match self {
            Self::Service { service_name, .. } => service_name.as_deref(),
            _ => None,
        }
    }

    /// Returns the AI provider for AI errors, if any.
    pub fn ai_provider(&self) -> Option<&str> {
        match self {
            Self::Ai { provider, .. } => provider.as_deref(),
            _ => None,
        }
    }

    /// Returns the URL for network errors, if any.
    pub fn url(&self) -> Option<&str> {
        match self {
            Self::Network { url, .. } => url.as_deref(),
            _ => None,
        }
    }

    /// Creates a configuration error for missing config file.
    pub fn config_file_not_found(path: impl Into<String>) -> Self {
        Self::config_error(
            format!("Configuration file '{}' not found", path.into()),
            Some("Create the configuration file or check the path"),
        )
    }

    /// Creates a configuration error for invalid YAML.
    pub fn config_invalid_yaml(error: impl fmt::Display) -> Self {
        Self::config_error(
            format!("Invalid YAML in configuration: {}", error),
            Some("Check YAML syntax and indentation"),
        )
    }

    /// Creates a service error for service not found.
    pub fn service_not_found(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::service_error(
            format!("Service '{}' not found", name),
            Some(name),
            Some("Check if the service is defined in your configuration"),
        )
    }

    /// Creates a service error for service already running.
    pub fn service_already_running(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::service_error(
            format!("Service '{}' is already running", name),
            Some(name),
            Some("Stop the service first or use a different name"),
        )
    }

    /// Creates an AI error for provider not configured.
    pub fn ai_provider_not_configured(provider: impl Into<String>) -> Self {
        let provider = provider.into();
        Self::ai_error(
            format!("AI provider '{}' not configured", provider),
            Some(provider),
            Some("Add AI provider configuration to apicentric.json"),
        )
    }

    /// Creates a recording error for no active session.
    pub fn recording_not_active() -> Self {
        Self::recording_error(
            "No active recording session",
            Some("Start a recording session before performing this action"),
        )
    }

    /// Creates an authentication error for invalid token.
    pub fn auth_invalid_token() -> Self {
        Self::auth_error(
            "Invalid authentication token",
            Some("Check your token or re-authenticate"),
        )
    }

    /// Creates a network error for connection failed.
    pub fn network_connection_failed(url: impl Into<String>) -> Self {
        Self::network_error(
            "Connection failed",
            Some(url.into()),
            Some("Check network connectivity and URL"),
        )
    }

    /// Creates a file system error for permission denied.
    pub fn fs_permission_denied(path: impl Into<String>) -> Self {
        Self::fs_error(
            format!("Permission denied accessing '{}'", path.into()),
            Some("Check file permissions or run with appropriate privileges"),
        )
    }

    /// Creates a validation error for required field missing.
    pub fn validation_required_field(field: impl Into<String>) -> Self {
        Self::validation_error(
            "This field is required",
            Some(field.into()),
            Some("Provide a value for this required field"),
        )
    }
}

impl From<std::io::Error> for ApicentricError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for ApicentricError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<glob::GlobError> for ApicentricError {
    fn from(err: glob::GlobError) -> Self {
        Self::Glob(err)
    }
}

impl From<glob::PatternError> for ApicentricError {
    fn from(err: glob::PatternError) -> Self {
        Self::Pattern(err)
    }
}

#[cfg(feature = "scripting")]
impl From<rhai::ParseError> for ApicentricError {
    fn from(err: rhai::ParseError) -> Self {
        Self::Scripting {
            message: format!("Rhai parse error: {}", err),
            suggestion: Some("Check script syntax".to_string()),
        }
    }
}

#[cfg(feature = "scripting")]
impl From<Box<rhai::EvalAltResult>> for ApicentricError {
    fn from(err: Box<rhai::EvalAltResult>) -> Self {
        Self::Scripting {
            message: format!("Rhai execution error: {}", err),
            suggestion: Some("Check runtime logic".to_string()),
        }
    }
}

#[cfg(feature = "iot")]
impl From<rumqttc::ClientError> for ApicentricError {
    fn from(err: rumqttc::ClientError) -> Self {
        Self::Mqtt {
            message: err.to_string(),
            suggestion: Some("Check MQTT broker connection details".to_string()),
        }
    }
}

#[cfg(feature = "iot")]
impl From<csv::Error> for ApicentricError {
    fn from(err: csv::Error) -> Self {
        Self::Csv {
            message: err.to_string(),
            suggestion: Some("Check CSV format".to_string()),
        }
    }
}

/// A `Result` type alias for Apicentric operations.
pub type ApicentricResult<T> = Result<T, ApicentricError>;

/// Represents a validation error with details about the field, message, and an
/// optional suggestion.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The name of the field that failed validation.
    pub field: String,
    /// The error message.
    pub message: String,
    /// An optional suggestion for how to fix the error.
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Creates a new validation error.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field that failed validation.
    /// * `message` - The error message.
    ///
    /// # Returns
    ///
    /// A new `ValidationError` instance.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Adds a suggestion to the validation error.
    ///
    /// # Arguments
    ///
    /// * `suggestion` - The suggestion to add.
    ///
    /// # Returns
    ///
    /// The `ValidationError` instance with the added suggestion.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// A utility for formatting errors for user-friendly display.
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Formats an error for user display with context and suggestions.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to format.
    ///
    /// # Returns
    ///
    /// A string containing the formatted error message.
    pub fn format_for_user(error: &ApicentricError) -> String {
        let mut output = format!("❌ {}", error);

        if let Some(suggestion) = error.suggestion() {
            output.push_str(&format!("\n💡 Suggestion: {}", suggestion));
        }

        if let Some(field) = error.field() {
            output.push_str(&format!("\n🔍 Field: {}", field));
        }

        if let Some(service_name) = error.service_name() {
            output.push_str(&format!("\n🏷️ Service: {}", service_name));
        }

        if let Some(provider) = error.ai_provider() {
            output.push_str(&format!("\n🤖 AI Provider: {}", provider));
        }

        if let Some(url) = error.url() {
            output.push_str(&format!("\n🌐 URL: {}", url));
        }

        output
    }

    /// Formats a slice of validation errors into a single string.
    ///
    /// # Arguments
    ///
    /// * `errors` - The validation errors to format.
    ///
    /// # Returns
    ///
    /// A string containing the formatted validation errors.
    pub fn format_validation_errors(errors: &[ValidationError]) -> String {
        let mut output = String::from("❌ Configuration validation failed:\n");

        for (i, error) in errors.iter().enumerate() {
            output.push_str(&format!(
                "  {}. Field '{}': {}",
                i + 1,
                error.field,
                error.message
            ));

            if let Some(suggestion) = &error.suggestion {
                output.push_str(&format!("\n     💡 {}", suggestion));
            }

            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = ApicentricError::config_error(
            "Invalid config",
            Some("Check your apicentric.json file"),
        );
        assert!(error.suggestion().is_some());
        assert_eq!(
            error.suggestion().unwrap(),
            "Check your apicentric.json file"
        );
    }

    #[test]
    fn test_new_error_variants() {
        let service_error = ApicentricError::service_not_found("test-service");
        assert!(matches!(service_error, ApicentricError::Service { .. }));
        assert_eq!(service_error.service_name().unwrap(), "test-service");
        assert!(service_error.suggestion().is_some());

        let ai_error = ApicentricError::ai_provider_not_configured("openai");
        assert!(matches!(ai_error, ApicentricError::Ai { .. }));
        assert_eq!(ai_error.ai_provider().unwrap(), "openai");

        let network_error = ApicentricError::network_connection_failed("http://example.com");
        assert!(matches!(network_error, ApicentricError::Network { .. }));
        assert_eq!(network_error.url().unwrap(), "http://example.com");
    }

    #[test]
    fn test_validation_error() {
        let error = ValidationError::new("base_url", "Invalid URL format")
            .with_suggestion("Use format: http://localhost:5173");

        assert_eq!(error.field, "base_url");
        assert_eq!(error.message, "Invalid URL format");
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn test_error_formatting() {
        let error = ApicentricError::validation_error(
            "Invalid configuration",
            Some("base_url"),
            Some("Use a valid URL format"),
        );

        let formatted = ErrorFormatter::format_for_user(&error);
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("💡 Suggestion"));
        assert!(formatted.contains("🔍 Field"));
    }

    #[test]
    fn test_all_error_types() {
        let config_error = ApicentricError::config_error("Config issue", None::<String>);
        assert!(matches!(
            config_error,
            ApicentricError::Configuration { .. }
        ));

        let server_error = ApicentricError::server_error("Server issue", Some("Restart server"));
        assert!(matches!(server_error, ApicentricError::Server { .. }));
        assert_eq!(server_error.suggestion().unwrap(), "Restart server");

        let test_error = ApicentricError::test_error("Test failed", Some("Check test file"));
        assert!(matches!(test_error, ApicentricError::TestExecution { .. }));

        let fs_error = ApicentricError::fs_error("File not found", Some("Create the file"));
        assert!(matches!(fs_error, ApicentricError::FileSystem { .. }));

        let validation_error =
            ApicentricError::validation_error("Invalid", Some("field"), Some("Fix it"));
        assert!(matches!(
            validation_error,
            ApicentricError::Validation { .. }
        ));
        assert_eq!(validation_error.field().unwrap(), "field");
    }

    #[test]
    fn test_error_from_conversions() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let apicentric_error: ApicentricError = io_error.into();
        assert!(matches!(apicentric_error, ApicentricError::Io(_)));

        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let apicentric_error: ApicentricError = json_error.into();
        assert!(matches!(apicentric_error, ApicentricError::Json(_)));
    }

    #[test]
    fn test_validation_error_builder() {
        let error = ValidationError::new("timeout", "Value too low")
            .with_suggestion("Use a value between 1000 and 300000");

        assert_eq!(error.field, "timeout");
        assert_eq!(error.message, "Value too low");
        assert_eq!(
            error.suggestion.as_ref().unwrap(),
            "Use a value between 1000 and 300000"
        );
    }

    #[test]
    fn test_error_formatter_multiple_validation_errors() {
        let errors = vec![
            ValidationError::new("base_url", "Invalid URL")
                .with_suggestion("Use http://localhost:5173"),
            ValidationError::new("timeout", "Too low").with_suggestion("Use at least 1000ms"),
            ValidationError::new("pattern", "Invalid glob"),
        ];

        let formatted = ErrorFormatter::format_validation_errors(&errors);

        assert!(formatted.contains("❌ Configuration validation failed"));
        assert!(formatted.contains("1. Field 'base_url'"));
        assert!(formatted.contains("2. Field 'timeout'"));
        assert!(formatted.contains("3. Field 'pattern'"));
        assert!(formatted.contains("💡 Use http://localhost:5173"));
        assert!(formatted.contains("💡 Use at least 1000ms"));
    }

    #[test]
    fn test_error_suggestion_and_field_getters() {
        let error = ApicentricError::validation_error(
            "Test message",
            Some("test_field"),
            Some("Test suggestion"),
        );

        assert_eq!(error.suggestion().unwrap(), "Test suggestion");
        assert_eq!(error.field().unwrap(), "test_field");

        let io_error: ApicentricError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "test").into();
        assert!(io_error.suggestion().is_none());
        assert!(io_error.field().is_none());
    }

    #[test]
    fn test_error_display() {
        let error = ApicentricError::config_error("Test config error", Some("Fix the config"));
        let display_str = format!("{}", error);
        assert!(display_str.contains("Configuration error: Test config error"));

        let formatted = ErrorFormatter::format_for_user(&error);
        assert!(formatted.contains("❌ Configuration error: Test config error"));
        assert!(formatted.contains("💡 Suggestion: Fix the config"));
    }

    #[test]
    fn test_new_error_display_and_formatting() {
        let service_error = ApicentricError::service_not_found("my-service");
        let display_str = format!("{}", service_error);
        assert!(display_str.contains("Service error for 'my-service'"));

        let formatted = ErrorFormatter::format_for_user(&service_error);
        assert!(formatted.contains("❌ Service error for 'my-service'"));
        assert!(formatted.contains("🏷️ Service: my-service"));
        assert!(formatted.contains("💡 Suggestion"));

        let ai_error = ApicentricError::ai_provider_not_configured("openai");
        let formatted_ai = ErrorFormatter::format_for_user(&ai_error);
        assert!(formatted_ai.contains("🤖 AI Provider: openai"));
    }
}
