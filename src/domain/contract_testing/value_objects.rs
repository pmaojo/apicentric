use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Unique identifier for a contract
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractId(String);

impl ContractId {
    pub fn new(id: String) -> Result<Self, ContractValidationError> {
        if id.is_empty() {
            return Err(ContractValidationError::EmptyContractId);
        }

        if id.len() > 100 {
            return Err(ContractValidationError::ContractIdTooLong);
        }

        // Allow alphanumeric, hyphens, and underscores
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ContractValidationError::InvalidContractIdFormat);
        }

        Ok(ContractId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// URL for API endpoints with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiUrl(String);

impl ApiUrl {
    pub fn new(url: String) -> Result<Self, ContractValidationError> {
        if url.is_empty() {
            return Err(ContractValidationError::EmptyApiUrl);
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ContractValidationError::InvalidApiUrlFormat);
        }

        Ok(ApiUrl(url))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ApiUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Timeout duration with constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutDuration(u64);

impl TimeoutDuration {
    pub fn new(millis: u64) -> Result<Self, ContractValidationError> {
        if millis == 0 {
            return Err(ContractValidationError::ZeroTimeout);
        }

        if millis > 300_000 {
            // 5 minutes max
            return Err(ContractValidationError::TimeoutTooLong);
        }

        Ok(TimeoutDuration(millis))
    }

    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

/// Retry attempts with constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryAttempts(u32);

impl RetryAttempts {
    pub fn new(attempts: u32) -> Result<Self, ContractValidationError> {
        if attempts > 10 {
            return Err(ContractValidationError::TooManyRetries);
        }

        Ok(RetryAttempts(attempts))
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// HTTP methods supported in contract testing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method_str = match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        };
        write!(f, "{}", method_str)
    }
}

/// Types of request/response body content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseBody {
    Json(serde_json::Value),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestBody {
    Json(serde_json::Value),
    Text(String),
    FormData(HashMap<String, String>),
}

/// Severity levels for compliance issues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Types of compliance issues that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceIssueType {
    StatusCodeMismatch,
    ResponseSchemaMismatch,
    BodyMismatch,
    HeaderMismatch,
    TimeoutExceeded,
    ValidationError,
    UnexpectedError,
}

// === ERROR TYPES ===
#[derive(Debug, Error)]
pub enum ContractValidationError {
    #[error("Contract ID cannot be empty")]
    EmptyContractId,

    #[error("Contract ID is too long (max 100 characters)")]
    ContractIdTooLong,

    #[error("Contract ID contains invalid characters (only alphanumeric, hyphens, and underscores allowed)")]
    InvalidContractIdFormat,

    #[error("API URL cannot be empty")]
    EmptyApiUrl,

    #[error("API URL must start with http:// or https://")]
    InvalidApiUrlFormat,

    #[error("Timeout cannot be zero")]
    ZeroTimeout,

    #[error("Timeout too long (max 5 minutes)")]
    TimeoutTooLong,

    #[error("Too many retry attempts (max 10)")]
    TooManyRetries,

    #[error("Service name cannot be empty")]
    EmptyServiceName,

    #[error("Specification path cannot be empty")]
    EmptySpecPath,

    #[error("Failed to load specification: {0}")]
    SpecLoadError(String),

    #[error("Specification validation failed: {0}")]
    SpecValidationError(String),

    #[error("Mock API error: {0}")]
    MockApiError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("HTTP client error: {0}")]
    HttpClientError(String),
}
