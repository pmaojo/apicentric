use thiserror::Error;

/// Main error type for Apicentric operations
#[derive(Debug, Error)]
pub enum ApicentricError {
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        suggestion: Option<String>,
    },

    #[error("Server error: {message}")]
    Server {
        message: String,
        suggestion: Option<String>,
    },

    #[error("Test execution error: {message}")]
    TestExecution {
        message: String,
        suggestion: Option<String>,
    },

    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        suggestion: Option<String>,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        suggestion: Option<String>,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::GlobError),

    #[error("Pattern error: {0}")]
    Pattern(#[from] glob::PatternError),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl ApicentricError {
    /// Create a configuration error with a suggestion
    pub fn config_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Configuration {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create a validation error with field and suggestion
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

    /// Create a file system error with suggestion
    pub fn fs_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::FileSystem {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create a server error with suggestion
    pub fn server_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Server {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create a test execution error with suggestion
    pub fn test_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::TestExecution {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create a runtime error with suggestion
    pub fn runtime_error(
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::Runtime {
            message: message.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Get the suggestion for this error, if any
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

    /// Get the field name for validation errors
    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Validation { field, .. } => field.as_deref(),
            _ => None,
        }
    }
}

/// Result type alias for Apicentric operations
pub type ApicentricResult<T> = Result<T, ApicentricError>;

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Error formatter for user-friendly error messages
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format an error for user display with context and suggestions
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

    /// Format multiple validation errors
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
