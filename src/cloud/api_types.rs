//! API types for the cloud module.

use serde::{Deserialize, Serialize};
use crate::simulator::config::EndpointDefinition;
use crate::simulator::ServiceInfo;

/// A generic API response.
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful.
    pub success: bool,
    /// The data returned by the request.
    pub data: Option<T>,
    /// An error message if the request was not successful.
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Creates a new successful `ApiResponse`.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to include in the response.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Creates a new error `ApiResponse`.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// A request to load a service.
#[derive(Deserialize)]
pub struct LoadServiceRequest {
    /// The path to the service definition file.
    pub path: String,
}

/// A request to save a service.
#[derive(Deserialize)]
pub struct SaveServiceRequest {
    /// The path to the service definition file.
    pub path: String,
    /// The YAML content of the service definition.
    pub yaml: String,
}

/// A request to create a new service.
#[derive(Deserialize)]
pub struct CreateServiceRequest {
    /// The YAML content of the service definition.
    pub yaml: String,
    /// Optional custom filename (defaults to service name from YAML).
    pub filename: Option<String>,
}

/// A request to create a new GraphQL service.
#[derive(Deserialize)]
pub struct CreateGraphQLServiceRequest {
    pub name: String,
    pub port: u16,
}

/// A request to update a service.
#[derive(Deserialize)]
pub struct UpdateServiceRequest {
    /// The YAML content of the service definition.
    pub yaml: String,
}

/// A query for logs.
#[derive(Deserialize)]
pub struct LogsQuery {
    /// The maximum number of logs to return.
    pub limit: Option<usize>,
    /// Filter by service name.
    pub service: Option<String>,
    /// Filter by HTTP method.
    pub method: Option<String>,
    /// Filter by status code.
    pub status: Option<u16>,
    /// Filter by route/path.
    pub route: Option<String>,
}

/// Export format for logs.
#[derive(Deserialize)]
pub struct LogsExportQuery {
    /// Export format (json or csv).
    pub format: Option<String>,
    /// Maximum number of logs to export.
    pub limit: Option<usize>,
}

/// Request to start recording mode.
#[derive(Deserialize)]
pub struct StartRecordingRequest {
    /// The target URL to proxy to.
    pub target_url: String,
    /// The port to listen on (optional, defaults to 8888).
    pub port: Option<u16>,
}

/// Response for starting recording mode.
#[derive(Serialize)]
pub struct StartRecordingResponse {
    /// The session ID for this recording.
    pub session_id: String,
    /// The proxy URL to use.
    pub proxy_url: String,
    /// The port the proxy is listening on.
    pub proxy_port: u16,
    /// The target URL being proxied.
    pub target_url: String,
}

/// Response for recording status.
#[derive(Serialize)]
pub struct RecordingStatusResponse {
    /// Whether recording is currently active.
    pub is_active: bool,
    /// The session ID if recording is active.
    pub session_id: Option<String>,
    /// The proxy URL if recording is active.
    pub proxy_url: Option<String>,
    /// The proxy port if recording is active.
    pub proxy_port: Option<u16>,
    /// The target URL if recording is active.
    pub target_url: Option<String>,
    /// Number of requests captured so far.
    pub captured_count: usize,
}

/// Response for stopping recording mode.
#[derive(Serialize)]
pub struct StopRecordingResponse {
    /// The session ID that was stopped.
    pub session_id: String,
    /// Number of requests captured.
    pub captured_count: usize,
    /// The captured endpoints.
    pub endpoints: Vec<EndpointDefinition>,
}

/// Request to generate a service from captured requests.
#[derive(Deserialize)]
pub struct GenerateServiceRequest {
    /// The name for the generated service.
    pub service_name: String,
    /// Optional description for the service.
    pub description: Option<String>,
}

/// Service status response.
#[derive(Serialize)]
pub struct ServiceStatusResponse {
    /// The name of the service.
    pub name: String,
    /// Whether the service is running.
    pub is_running: bool,
    /// The port the service is running on.
    pub port: Option<u16>,
    /// The number of endpoints.
    pub endpoint_count: usize,
}

/// Detailed service response with full definition.
#[derive(Serialize)]
pub struct ServiceDetailResponse {
    /// The service information.
    pub info: ServiceInfo,
    /// The YAML definition.
    pub yaml: String,
}

/// Request to generate a service using AI.
#[derive(Deserialize)]
pub struct AiGenerateRequest {
    /// The natural language prompt describing the service.
    pub prompt: String,
    /// Optional AI provider to use (openai, gemini, local).
    pub provider: Option<String>,
}

/// Response from AI generation.
#[derive(Serialize)]
pub struct AiGenerateResponse {
    /// The generated YAML service definition.
    pub yaml: String,
    /// Any validation errors found in the generated YAML.
    pub validation_errors: Vec<String>,
}

/// Request to validate YAML.
#[derive(Deserialize)]
pub struct AiValidateRequest {
    /// The YAML content to validate.
    pub yaml: String,
}

/// Response from YAML validation.
#[derive(Serialize)]
pub struct AiValidateResponse {
    /// Whether the YAML is valid.
    pub is_valid: bool,
    /// Any validation errors found.
    pub errors: Vec<String>,
}

/// Response for AI configuration status.
#[derive(Serialize)]
pub struct AiConfigResponse {
    /// Whether AI is configured.
    pub is_configured: bool,
    /// The configured provider (if any).
    pub provider: Option<String>,
    /// The configured model (if any).
    pub model: Option<String>,
    /// Any configuration issues.
    pub issues: Vec<String>,
}

/// Request to generate TypeScript types.
#[derive(Deserialize)]
pub struct TypeScriptGenerateRequest {
    /// The name of the service to generate types for.
    pub service_name: String,
}

/// Response from TypeScript generation.
#[derive(Serialize)]
pub struct TypeScriptGenerateResponse {
    /// The generated TypeScript code.
    pub code: String,
}

/// Request to generate React Query hooks.
#[derive(Deserialize)]
pub struct ReactQueryGenerateRequest {
    /// The name of the service to generate hooks for.
    pub service_name: String,
}

/// Response from React Query generation.
#[derive(Serialize)]
pub struct ReactQueryGenerateResponse {
    /// The generated React Query hooks code.
    pub code: String,
}

/// Request to generate Axios client.
#[derive(Deserialize)]
pub struct AxiosGenerateRequest {
    /// The name of the service to generate client for.
    pub service_name: String,
}

/// Response from Axios client generation.
#[derive(Serialize)]
pub struct AxiosGenerateResponse {
    /// The generated Axios client code.
    pub code: String,
}

/// Request to update configuration.
#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    /// The configuration as JSON.
    pub config: serde_json::Value,
}

/// Response from configuration validation.
#[derive(Serialize)]
pub struct ValidateConfigResponse {
    /// Whether the configuration is valid.
    pub is_valid: bool,
    /// Any validation errors found.
    pub errors: Vec<String>,
}

/// Response for metrics endpoint
#[derive(Serialize)]
pub struct MetricsResponse {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Number of active WebSocket connections
    pub active_websocket_connections: u64,
    /// Number of active services
    pub active_services: u64,
    /// Total number of log entries
    pub total_log_entries: u64,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<u64>,
}

/// Request to import from URL
#[derive(Deserialize)]
pub struct ImportUrlRequest {
    pub url: String,
    pub format: Option<String>, // "openapi", "wiremock", etc.
}

/// Response for import
#[derive(Serialize)]
pub struct ImportUrlResponse {
    pub service_name: String,
    pub yaml: String,
}

/// Response for the legacy /status endpoint
#[derive(Serialize)]
pub struct LegacySimulatorStatus {
    pub active_services: Vec<LegacyServiceInfo>,
    pub is_running: bool,
}

#[derive(Serialize)]
pub struct LegacyServiceInfo {
    pub name: String,
    pub port: u16,
    pub is_running: bool,
    pub endpoints_count: usize,
}
