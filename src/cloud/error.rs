//! Error handling for the cloud API.
//!
//! This module provides comprehensive error types, error codes, and response models
//! for consistent error handling across all API endpoints.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard error codes for API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiErrorCode {
    // Service errors
    ServiceNotFound,
    ServiceAlreadyExists,
    ServiceAlreadyRunning,
    ServiceNotRunning,
    ServiceStartFailed,
    ServiceStopFailed,

    // File system errors
    FileNotFound,
    FileAlreadyExists,
    FileReadError,
    FileWriteError,
    DirectoryCreateError,

    // Validation errors
    InvalidYaml,
    InvalidServiceName,
    InvalidConfiguration,
    ValidationFailed,
    YamlTooLarge,
    ServiceNameMismatch,

    // Recording errors
    RecordingNotActive,
    RecordingAlreadyActive,
    RecordingStartFailed,
    RecordingStopFailed,
    NoRequestsCaptured,

    // AI errors
    AiNotConfigured,
    AiGenerationFailed,
    AiProviderError,
    InvalidAiProvider,

    // Code generation errors
    CodeGenerationFailed,

    // Configuration errors
    ConfigLoadError,
    ConfigSaveError,
    ConfigValidationError,

    // Authentication errors
    AuthenticationRequired,
    InvalidToken,
    TokenExpired,

    // General errors
    InternalError,
    InvalidRequest,
    InvalidParameter,
    MissingParameter,
}

impl fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ServiceNotFound => write!(f, "SERVICE_NOT_FOUND"),
            Self::ServiceAlreadyExists => write!(f, "SERVICE_ALREADY_EXISTS"),
            Self::ServiceAlreadyRunning => write!(f, "SERVICE_ALREADY_RUNNING"),
            Self::ServiceNotRunning => write!(f, "SERVICE_NOT_RUNNING"),
            Self::ServiceStartFailed => write!(f, "SERVICE_START_FAILED"),
            Self::ServiceStopFailed => write!(f, "SERVICE_STOP_FAILED"),
            Self::FileNotFound => write!(f, "FILE_NOT_FOUND"),
            Self::FileAlreadyExists => write!(f, "FILE_ALREADY_EXISTS"),
            Self::FileReadError => write!(f, "FILE_READ_ERROR"),
            Self::FileWriteError => write!(f, "FILE_WRITE_ERROR"),
            Self::DirectoryCreateError => write!(f, "DIRECTORY_CREATE_ERROR"),
            Self::InvalidYaml => write!(f, "INVALID_YAML"),
            Self::InvalidServiceName => write!(f, "INVALID_SERVICE_NAME"),
            Self::InvalidConfiguration => write!(f, "INVALID_CONFIGURATION"),
            Self::ValidationFailed => write!(f, "VALIDATION_FAILED"),
            Self::YamlTooLarge => write!(f, "YAML_TOO_LARGE"),
            Self::ServiceNameMismatch => write!(f, "SERVICE_NAME_MISMATCH"),
            Self::RecordingNotActive => write!(f, "RECORDING_NOT_ACTIVE"),
            Self::RecordingAlreadyActive => write!(f, "RECORDING_ALREADY_ACTIVE"),
            Self::RecordingStartFailed => write!(f, "RECORDING_START_FAILED"),
            Self::RecordingStopFailed => write!(f, "RECORDING_STOP_FAILED"),
            Self::NoRequestsCaptured => write!(f, "NO_REQUESTS_CAPTURED"),
            Self::AiNotConfigured => write!(f, "AI_NOT_CONFIGURED"),
            Self::AiGenerationFailed => write!(f, "AI_GENERATION_FAILED"),
            Self::AiProviderError => write!(f, "AI_PROVIDER_ERROR"),
            Self::InvalidAiProvider => write!(f, "INVALID_AI_PROVIDER"),
            Self::CodeGenerationFailed => write!(f, "CODE_GENERATION_FAILED"),
            Self::ConfigLoadError => write!(f, "CONFIG_LOAD_ERROR"),
            Self::ConfigSaveError => write!(f, "CONFIG_SAVE_ERROR"),
            Self::ConfigValidationError => write!(f, "CONFIG_VALIDATION_ERROR"),
            Self::AuthenticationRequired => write!(f, "AUTHENTICATION_REQUIRED"),
            Self::InvalidToken => write!(f, "INVALID_TOKEN"),
            Self::TokenExpired => write!(f, "TOKEN_EXPIRED"),
            Self::InternalError => write!(f, "INTERNAL_ERROR"),
            Self::InvalidRequest => write!(f, "INVALID_REQUEST"),
            Self::InvalidParameter => write!(f, "INVALID_PARAMETER"),
            Self::MissingParameter => write!(f, "MISSING_PARAMETER"),
        }
    }
}

/// Standard error response structure for API endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Whether the request was successful (always false for errors).
    pub success: bool,
    /// The error code.
    pub code: ApiErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional details about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Creates a new error response.
    pub fn new(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            success: false,
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Adds details to the error response.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Creates a service not found error.
    pub fn service_not_found(service_name: &str) -> Self {
        Self::new(
            ApiErrorCode::ServiceNotFound,
            format!("Service '{}' not found", service_name),
        )
    }

    /// Creates a service already exists error.
    pub fn service_already_exists(service_name: &str) -> Self {
        Self::new(
            ApiErrorCode::ServiceAlreadyExists,
            format!("Service '{}' already exists", service_name),
        )
    }

    /// Creates a service already running error.
    pub fn service_already_running(service_name: &str) -> Self {
        Self::new(
            ApiErrorCode::ServiceAlreadyRunning,
            format!("Service '{}' is already running", service_name),
        )
    }

    /// Creates an invalid YAML error.
    pub fn invalid_yaml(error: impl fmt::Display) -> Self {
        Self::new(
            ApiErrorCode::InvalidYaml,
            format!("Invalid YAML: {}", error),
        )
    }

    /// Creates an invalid service name error.
    pub fn invalid_service_name(name: &str, reason: &str) -> Self {
        Self::new(
            ApiErrorCode::InvalidServiceName,
            format!("Invalid service name '{}': {}", name, reason),
        )
    }

    /// Creates a YAML too large error.
    pub fn yaml_too_large(size: usize, max_size: usize) -> Self {
        Self::new(
            ApiErrorCode::YamlTooLarge,
            format!(
                "YAML size ({} bytes) exceeds maximum allowed size ({} bytes)",
                size, max_size
            ),
        )
    }

    /// Creates a validation failed error.
    pub fn validation_failed(errors: Vec<String>) -> Self {
        Self::new(
            ApiErrorCode::ValidationFailed,
            "Configuration validation failed",
        )
        .with_details(serde_json::json!({ "errors": errors }))
    }

    /// Creates an AI not configured error.
    pub fn ai_not_configured() -> Self {
        Self::new(
            ApiErrorCode::AiNotConfigured,
            "AI provider not configured. Add an 'ai' section to apicentric.json",
        )
    }

    /// Creates a recording not active error.
    pub fn recording_not_active() -> Self {
        Self::new(
            ApiErrorCode::RecordingNotActive,
            "No active recording session",
        )
    }

    /// Creates an internal error.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(ApiErrorCode::InternalError, message)
    }
}

/// API error type that can be converted into an HTTP response.
#[derive(Debug)]
pub struct ApiError {
    /// HTTP status code.
    pub status: StatusCode,
    /// Error response body.
    pub response: ErrorResponse,
}

impl ApiError {
    /// Creates a new API error.
    pub fn new(status: StatusCode, code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            response: ErrorResponse::new(code, message),
        }
    }

    /// Adds details to the error.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.response = self.response.with_details(details);
        self
    }

    /// Creates a 404 Not Found error.
    pub fn not_found(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message)
    }

    /// Creates a 400 Bad Request error.
    pub fn bad_request(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    /// Creates a 409 Conflict error.
    pub fn conflict(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    /// Creates a 500 Internal Server Error.
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::InternalError,
            message,
        )
    }

    /// Creates a 401 Unauthorized error.
    pub fn unauthorized(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.response)).into_response()
    }
}

impl From<ErrorResponse> for ApiError {
    fn from(response: ErrorResponse) -> Self {
        // Map error codes to appropriate HTTP status codes
        let status = match response.code {
            ApiErrorCode::ServiceNotFound | ApiErrorCode::FileNotFound => StatusCode::NOT_FOUND,
            ApiErrorCode::ServiceAlreadyExists
            | ApiErrorCode::FileAlreadyExists
            | ApiErrorCode::ServiceAlreadyRunning
            | ApiErrorCode::RecordingAlreadyActive => StatusCode::CONFLICT,
            ApiErrorCode::InvalidYaml
            | ApiErrorCode::InvalidServiceName
            | ApiErrorCode::InvalidConfiguration
            | ApiErrorCode::ValidationFailed
            | ApiErrorCode::YamlTooLarge
            | ApiErrorCode::ServiceNameMismatch
            | ApiErrorCode::InvalidRequest
            | ApiErrorCode::InvalidParameter
            | ApiErrorCode::MissingParameter
            | ApiErrorCode::InvalidAiProvider => StatusCode::BAD_REQUEST,
            ApiErrorCode::AuthenticationRequired
            | ApiErrorCode::InvalidToken
            | ApiErrorCode::TokenExpired => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self { status, response }
    }
}

/// Input validation utilities.
pub mod validation {
    use super::*;

    /// Maximum YAML size in bytes (10 MB).
    pub const MAX_YAML_SIZE: usize = 10 * 1024 * 1024;

    /// Maximum service name length.
    pub const MAX_SERVICE_NAME_LENGTH: usize = 100;

    /// Validates a service name.
    pub fn validate_service_name(name: &str) -> Result<(), ErrorResponse> {
        if name.is_empty() {
            return Err(ErrorResponse::invalid_service_name(
                name,
                "name cannot be empty",
            ));
        }

        if name.len() > MAX_SERVICE_NAME_LENGTH {
            return Err(ErrorResponse::invalid_service_name(
                name,
                &format!(
                    "name exceeds maximum length of {} characters",
                    MAX_SERVICE_NAME_LENGTH
                ),
            ));
        }

        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ErrorResponse::invalid_service_name(
                name,
                "name can only contain alphanumeric characters, hyphens, and underscores",
            ));
        }

        // Check for path traversal attempts
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(ErrorResponse::invalid_service_name(
                name,
                "name cannot contain path separators or parent directory references",
            ));
        }

        Ok(())
    }

    /// Validates YAML content size.
    pub fn validate_yaml_size(yaml: &str) -> Result<(), ErrorResponse> {
        let size = yaml.len();
        if size > MAX_YAML_SIZE {
            return Err(ErrorResponse::yaml_too_large(size, MAX_YAML_SIZE));
        }
        Ok(())
    }

    /// Validates that a parameter is present.
    pub fn validate_required_param<T>(
        param: Option<T>,
        param_name: &str,
    ) -> Result<T, ErrorResponse> {
        param.ok_or_else(|| {
            ErrorResponse::new(
                ApiErrorCode::MissingParameter,
                format!("Required parameter '{}' is missing", param_name),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(
            ApiErrorCode::ServiceNotFound.to_string(),
            "SERVICE_NOT_FOUND"
        );
        assert_eq!(ApiErrorCode::InvalidYaml.to_string(), "INVALID_YAML");
        assert_eq!(
            ApiErrorCode::AiNotConfigured.to_string(),
            "AI_NOT_CONFIGURED"
        );
    }

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::service_not_found("test-service");
        assert_eq!(error.code, ApiErrorCode::ServiceNotFound);
        assert!(error.message.contains("test-service"));
        assert!(!error.success);
    }

    #[test]
    fn test_error_response_with_details() {
        let error = ErrorResponse::validation_failed(vec![
            "Field 'name' is required".to_string(),
            "Field 'port' must be between 1 and 65535".to_string(),
        ]);
        assert_eq!(error.code, ApiErrorCode::ValidationFailed);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_api_error_status_mapping() {
        let error: ApiError = ErrorResponse::service_not_found("test").into();
        assert_eq!(error.status, StatusCode::NOT_FOUND);

        let error: ApiError = ErrorResponse::invalid_yaml("bad yaml").into();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);

        let error: ApiError = ErrorResponse::service_already_exists("test").into();
        assert_eq!(error.status, StatusCode::CONFLICT);
    }

    #[test]
    fn test_validate_service_name() {
        use validation::*;

        // Valid names
        assert!(validate_service_name("my-service").is_ok());
        assert!(validate_service_name("service_123").is_ok());
        assert!(validate_service_name("MyService").is_ok());

        // Invalid names
        assert!(validate_service_name("").is_err());
        assert!(validate_service_name("my/service").is_err());
        assert!(validate_service_name("../service").is_err());
        assert!(validate_service_name("service with spaces").is_err());
        assert!(validate_service_name(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_yaml_size() {
        use validation::*;

        let small_yaml = "name: test\nversion: 1.0.0";
        assert!(validate_yaml_size(small_yaml).is_ok());

        let large_yaml = "a".repeat(MAX_YAML_SIZE + 1);
        assert!(validate_yaml_size(&large_yaml).is_err());
    }

    #[test]
    fn test_validate_required_param() {
        use validation::*;

        assert!(validate_required_param(Some("value"), "param").is_ok());
        assert!(validate_required_param(None::<String>, "param").is_err());
    }
}
