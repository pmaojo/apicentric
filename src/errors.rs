//! Defines the error types and error handling utilities for the Apicentric application.
//!
//! This module includes the main `ApicentricError` enum, which represents all possible
//! errors that can occur within the application. It also provides a custom `Result`
//! type alias, `ApicentricResult`, and an `ErrorFormatter` for creating user-friendly
//! error messages.

use thiserror::Error;

/// The main error type for all Apicentric operations.
///
/// This enum consolidates all possible errors that can occur within the application,
/// including configuration errors, server errors, file system issues, and more.
/// It uses `thiserror` to derive the `Error` trait and provide descriptive error
/// messages.
#[derive(Debug, Error)]
pub enum ApicentricError {
    /// An error related to application configuration.
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        suggestion: Option<String>,
    },

    /// An error that occurred within the server.
    #[error("Server error: {message}")]
    Server {
        message: String,
        suggestion: Option<String>,
    },

    /// An error that occurred during test execution.
    #[error("Test execution error: {message}")]
    TestExecution {
        message: String,
        suggestion: Option<String>,
    },

    /// An error related to file system operations.
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        suggestion: Option<String>,
    },

    /// An error that occurred during data validation.
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },

    /// A general-purpose runtime error.
    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        suggestion: Option<String>,
    },

    /// An I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// An error that occurred during JSON serialization or deserialization.
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// An error related to glob pattern matching.
    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::GlobError),

    /// An error in a glob pattern.
    #[error("Pattern error: {0}")]
    Pattern(#[from] glob::PatternError),

    /// An error from the `anyhow` crate.
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl ApicentricError {
    /// Creates a new configuration error.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `suggestion` - An optional suggestion for how to fix the error.
    ///
    /// # Returns
    ///
    /// A new `ApicentricError` instance.
    pub fn config_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Configuration {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new validation error.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `field` - The name of the field that failed validation.
    /// * `suggestion` - An optional suggestion for how to fix the error.
    ///
    /// # Returns
    ///
    /// A new `ApicentricError` instance.
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
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `suggestion` - An optional suggestion for how to fix the error.
    ///
    /// # Returns
    ///
    /// A new `ApicentricError` instance.
    pub fn fs_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::FileSystem {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new server error.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `suggestion` - An optional suggestion for how to fix the error.
    ///
    /// # Returns
    ///
    /// A new `ApicentricError` instance.
    pub fn server_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Server {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new test execution error.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `suggestion` - An optional suggestion for how to fix the error.
    ///
    /// # Returns
    ///
    /// A new `ApicentricError` instance.
    pub fn test_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::TestExecution {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Creates a new runtime error.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `suggestion` - An optional suggestion for how to fix the error.
    ///
    /// # Returns
    ///
    /// A new `ApicentricError` instance.
    pub fn runtime_error(
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Runtime {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Returns the suggestion for this error, if any.
    ///
    /// # Returns
    ///
    /// An `Option` containing the suggestion string, or `None` if there is no
    /// suggestion.
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::Configuration { suggestion, .. }
            | Self::Server { suggestion, .. }
            | Self::TestExecution { suggestion, .. }
            | Self::FileSystem { suggestion, .. }
            | Self::Validation { suggestion, .. }
            | Self::Runtime { suggestion, .. } => suggestion.as_deref(),
            _ => None,
        }
    }

    /// Returns the field name for validation errors, if any.
    ///
    /// # Returns
    ///
    /// An `Option` containing the field name, or `None` if the error is not a
    /// validation error.
    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Validation { field, .. } => field.as_deref(),
            _ => None,
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
        let error = ApicentricError::config_error("Invalid config", Some("Check your apicentric.json file"));
        assert!(error.suggestion().is_some());
        assert_eq!(error.suggestion().unwrap(), "Check your apicentric.json file");
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
        assert!(matches!(config_error, ApicentricError::Configuration { .. }));

        let server_error = ApicentricError::server_error("Server issue", Some("Restart server"));
        assert!(matches!(server_error, ApicentricError::Server { .. }));
        assert_eq!(server_error.suggestion().unwrap(), "Restart server");

        let test_error = ApicentricError::test_error("Test failed", Some("Check test file"));
        assert!(matches!(test_error, ApicentricError::TestExecution { .. }));

        let fs_error = ApicentricError::fs_error("File not found", Some("Create the file"));
        assert!(matches!(fs_error, ApicentricError::FileSystem { .. }));

        let validation_error =
            ApicentricError::validation_error("Invalid", Some("field"), Some("Fix it"));
        assert!(matches!(validation_error, ApicentricError::Validation { .. }));
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

        let io_error: ApicentricError = std::io::Error::new(std::io::ErrorKind::NotFound, "test").into();
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
}
