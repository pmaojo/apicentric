use crate::errors::ValidationError;
use crate::validation::{ConfigValidator, ValidationUtils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type of endpoint supported by the simulator
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum EndpointKind {
    /// Standard HTTP request/response endpoint
    #[default]
    Http,
    /// WebSocket endpoint
    #[serde(alias = "ws")]
    WebSocket,
    /// Server Sent Events endpoint
    #[serde(alias = "sse")]
    Sse,
}

/// Configuration for streaming style endpoints (WebSocket/SSE)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct StreamConfig {
    /// Messages sent immediately after connection establishment
    #[serde(default)]
    pub initial: Vec<String>,
    /// Periodic message configuration
    #[serde(default)]
    pub periodic: Option<PeriodicMessage>,
}

/// Configuration for a periodic message
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PeriodicMessage {
    /// Interval in milliseconds between messages
    pub interval_ms: u64,
    /// Message template rendered each interval
    pub message: String,
}

/// Endpoint definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EndpointDefinition {
    /// Type of endpoint (HTTP/WebSocket/SSE)
    #[serde(default)]
    pub kind: EndpointKind,
    pub method: String,
    pub path: String,
    /// Optional headers that must match for this endpoint to trigger
    #[serde(default)]
    pub header_match: Option<HashMap<String, String>>,
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Option<Vec<ParameterDefinition>>,
    #[serde(default)]
    pub request_body: Option<RequestBodyDefinition>,
    pub responses: HashMap<u16, ResponseDefinition>,
    /// Optional scenario-based responses with matching conditions
    #[serde(default)]
    pub scenarios: Option<Vec<ScenarioDefinition>>,
    /// Streaming configuration for WebSocket/SSE endpoints
    #[serde(default)]
    pub stream: Option<StreamConfig>,
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
    pub schema: Option<String>, // Reference to model name
    #[serde(default)]
    pub script: Option<PathBuf>,
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

/// Scenario definition for conditional responses
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScenarioDefinition {
    /// Optional scenario name for manual selection
    pub name: Option<String>,
    /// Conditions that must be satisfied for this scenario
    #[serde(default)]
    pub conditions: Option<ScenarioConditions>,
    /// Response to return when this scenario matches
    pub response: ScenarioResponse,
    /// Strategy for selecting this scenario when multiple are available
    #[serde(default)]
    pub strategy: Option<ScenarioStrategy>,
}

/// Strategy for auto-selecting scenarios
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScenarioStrategy {
    Sequential,
    Random,
}

/// Conditions evaluated against incoming requests
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScenarioConditions {
    #[serde(default)]
    pub query: Option<HashMap<String, String>>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<HashMap<String, serde_json::Value>>,
}

/// Response associated with a scenario
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScenarioResponse {
    /// HTTP status code for the response
    pub status: u16,
    #[serde(flatten)]
    pub definition: ResponseDefinition,
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
            match status_code {
                204 | 304 => {}
                _ => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
