//! Axum handlers for the cloud API.
//!
//! This module provides handlers for listing, loading, and saving services.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::error::{validation, ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::recording_session::RecordingSessionManager;
use crate::simulator::config::{EndpointDefinition, ServerConfig};
use crate::simulator::log::RequestLogEntry;
use crate::simulator::{ApiSimulatorManager, ServiceDefinition, ServiceInfo};
use crate::validation::ConfigValidator;

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

/// Lists all active services.
///
/// # Arguments
///
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn list_services(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<Vec<ServiceInfo>>>, StatusCode> {
    let status = simulator.get_status().await;
    Ok(Json(ApiResponse::success(status.active_services)))
}

/// Loads a service definition from a file.
///
/// # Arguments
///
/// * `request` - The request to load the service.
#[axum::debug_handler]
pub async fn load_service(
    Json(request): Json<LoadServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match std::fs::File::open(&request.path) {
        Ok(file) => match serde_yaml::from_reader::<_, ServiceDefinition>(file) {
            Ok(def) => match serde_yaml::to_string(&def) {
                Ok(yaml) => Ok(Json(ApiResponse::success(yaml))),
                Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
            },
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

/// Saves a service definition to a file.
///
/// # Arguments
///
/// * `request` - The request to save the service.
#[axum::debug_handler]
pub async fn save_service(
    Json(request): Json<SaveServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match serde_yaml::from_str::<ServiceDefinition>(&request.yaml) {
        Ok(def) => match std::fs::File::create(&request.path) {
            Ok(file) => match serde_yaml::to_writer(file, &def) {
                Ok(_) => Ok(Json(ApiResponse::success("Service saved".to_string()))),
                Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
            },
            Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
        },
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

/// Starts a specific service by name.
///
/// # Arguments
///
/// * `name` - The name of the service to start.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn start_service(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    match simulator.start_service(&name).await {
        Ok(_) => Ok(Json(ApiResponse::success(format!(
            "Service '{}' started successfully",
            name
        )))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Err(ErrorResponse::service_not_found(&name).into())
            } else if error_msg.contains("already running") {
                Err(ErrorResponse::service_already_running(&name).into())
            } else {
                Err(ApiError::internal_server_error(format!(
                    "Failed to start service: {}",
                    error_msg
                )))
            }
        }
    }
}

/// Stops a specific service by name.
///
/// # Arguments
///
/// * `name` - The name of the service to stop.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn stop_service(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    match simulator.stop_service(&name).await {
        Ok(_) => Ok(Json(ApiResponse::success(format!(
            "Service '{}' stopped successfully",
            name
        )))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Err(ErrorResponse::service_not_found(&name).into())
            } else {
                Err(ApiError::internal_server_error(format!(
                    "Failed to stop service: {}",
                    error_msg
                )))
            }
        }
    }
}

/// Reloads all service configurations.
///
/// # Arguments
///
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn reload_services(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match simulator.reload_services().await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Services reloaded successfully".to_string(),
        ))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
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

/// Gets the detailed status of a specific service.
///
/// # Arguments
///
/// * `name` - The name of the service.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn get_service_status(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<ServiceStatusResponse>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    let registry = simulator.service_registry().read().await;

    if let Some(service_arc) = registry.get_service(&name) {
        let service = service_arc.read().await;
        let response = ServiceStatusResponse {
            name: name.clone(),
            is_running: service.is_running(),
            port: if service.is_running() {
                Some(service.port())
            } else {
                None
            },
            endpoint_count: service.definition().endpoints.len(),
        };
        Ok(Json(ApiResponse::success(response)))
    } else {
        Err(ErrorResponse::service_not_found(&name).into())
    }
}

/// Detailed service response with full definition.
#[derive(Serialize)]
pub struct ServiceDetailResponse {
    /// The service information.
    pub info: ServiceInfo,
    /// The YAML definition.
    pub yaml: String,
}

/// Gets the complete details of a specific service including its definition.
///
/// # Arguments
///
/// * `name` - The name of the service.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn get_service(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<ServiceDetailResponse>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    let registry = simulator.service_registry().read().await;

    if let Some(service_arc) = registry.get_service(&name) {
        let service = service_arc.read().await;
        let definition = service.definition();

        match serde_yaml::to_string(&definition) {
            Ok(yaml) => {
                let info = ServiceInfo {
                    name: definition.name.clone(),
                    port: service.port(),
                    base_path: definition.server.base_path.clone(),
                    endpoints_count: definition.endpoints.len(),
                    is_running: service.is_running(),
                };

                let response = ServiceDetailResponse { info, yaml };
                Ok(Json(ApiResponse::success(response)))
            }
            Err(e) => Err(ApiError::internal_server_error(format!(
                "Failed to serialize service: {}",
                e
            ))),
        }
    } else {
        Err(ErrorResponse::service_not_found(&name).into())
    }
}

/// Creates a new service from a YAML definition.
///
/// # Arguments
///
/// * `request` - The request containing the service YAML.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn create_service(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<CreateServiceRequest>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Validate YAML size
    validation::validate_yaml_size(&request.yaml).map_err(ApiError::from)?;

    // Parse the YAML to get the service name
    let definition: ServiceDefinition = match serde_yaml::from_str(&request.yaml) {
        Ok(def) => def,
        Err(e) => return Err(ErrorResponse::invalid_yaml(e).into()),
    };

    // Validate service name
    validation::validate_service_name(&definition.name).map_err(ApiError::from)?;

    // Determine the filename
    let filename = request
        .filename
        .unwrap_or_else(|| format!("{}.yaml", definition.name));
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    let file_path = std::path::Path::new(&services_dir).join(&filename);

    // Check if file already exists
    if file_path.exists() {
        return Err(ErrorResponse::service_already_exists(&filename).into());
    }

    // Create services directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&services_dir) {
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::DirectoryCreateError,
            format!("Failed to create services directory: {}", e),
        ));
    }

    // Write the service definition to file
    match std::fs::write(&file_path, &request.yaml) {
        Ok(_) => {
            // Apply the service to the running simulator
            if let Err(e) = simulator.apply_service_definition(definition).await {
                // Clean up the file if applying fails
                let _ = std::fs::remove_file(&file_path);
                return Err(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorCode::ServiceStartFailed,
                    format!("Failed to apply service: {}", e),
                ));
            }

            Ok(Json(ApiResponse::success(format!(
                "Service '{}' created successfully",
                filename
            ))))
        }
        Err(e) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::FileWriteError,
            format!("Failed to write service file: {}", e),
        )),
    }
}

/// Updates an existing service definition.
///
/// # Arguments
///
/// * `name` - The name of the service to update.
/// * `request` - The request containing the updated YAML.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn update_service(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<UpdateServiceRequest>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    // Validate YAML size
    validation::validate_yaml_size(&request.yaml).map_err(ApiError::from)?;

    // Parse the YAML
    let definition: ServiceDefinition = match serde_yaml::from_str(&request.yaml) {
        Ok(def) => def,
        Err(e) => return Err(ErrorResponse::invalid_yaml(e).into()),
    };

    // Verify the service name matches
    if definition.name != name {
        return Err(ApiError::bad_request(
            ApiErrorCode::ServiceNameMismatch,
            format!(
                "Service name mismatch: expected '{}', got '{}'",
                name, definition.name
            ),
        ));
    }

    // Find the service file
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    let file_path = std::path::Path::new(&services_dir).join(format!("{}.yaml", name));

    if !file_path.exists() {
        return Err(ErrorResponse::service_not_found(&name).into());
    }

    // Write the updated definition
    match std::fs::write(&file_path, &request.yaml) {
        Ok(_) => {
            // Apply the updated service to the running simulator
            if let Err(e) = simulator.apply_service_definition(definition).await {
                return Err(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorCode::ServiceStartFailed,
                    format!("Failed to apply updated service: {}", e),
                ));
            }

            Ok(Json(ApiResponse::success(format!(
                "Service '{}' updated successfully",
                name
            ))))
        }
        Err(e) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::FileWriteError,
            format!("Failed to write service file: {}", e),
        )),
    }
}

/// Deletes a service and its definition file.
///
/// # Arguments
///
/// * `name` - The name of the service to delete.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn delete_service(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    // Stop the service if it's running
    let _ = simulator.stop_service(&name).await;

    // Find and delete the service file
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    let file_path = std::path::Path::new(&services_dir).join(format!("{}.yaml", name));

    if !file_path.exists() {
        return Err(ErrorResponse::service_not_found(&name).into());
    }

    match std::fs::remove_file(&file_path) {
        Ok(_) => Ok(Json(ApiResponse::success(format!(
            "Service '{}' deleted successfully",
            name
        )))),
        Err(e) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::FileWriteError,
            format!("Failed to delete service file: {}", e),
        )),
    }
}

/// Queries request logs with optional filtering.
///
/// # Arguments
///
/// * `query` - The query parameters for filtering logs.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn query_logs(
    Query(query): Query<LogsQuery>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<Vec<RequestLogEntry>>>, StatusCode> {
    let registry = simulator.service_registry().read().await;
    let storage = registry.storage();

    let limit = query.limit.unwrap_or(100).min(1000); // Cap at 1000 entries

    let logs = storage
        .query_logs(
            query.service.as_deref(),
            query.route.as_deref(),
            query.method.as_deref(),
            query.status,
            limit,
        )
        .unwrap_or_default();

    Ok(Json(ApiResponse::success(logs)))
}

/// Clears all request logs.
///
/// # Arguments
///
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn clear_logs(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let registry = simulator.service_registry().read().await;
    let storage = registry.storage();

    match storage.clear_logs() {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Logs cleared successfully".to_string(),
        ))),
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to clear logs: {}",
            e
        )))),
    }
}

/// Exports request logs in JSON or CSV format.
///
/// # Arguments
///
/// * `query` - The export query parameters.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn export_logs(
    Query(query): Query<LogsExportQuery>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<axum::response::Response, StatusCode> {
    let registry = simulator.service_registry().read().await;
    let storage = registry.storage();

    let limit = query.limit.unwrap_or(1000).min(10000); // Cap at 10000 for export
    let logs = storage
        .query_logs(None, None, None, None, limit)
        .unwrap_or_default();

    let format = query.format.as_deref().unwrap_or("json");

    match format {
        "csv" => {
            // Generate CSV
            let mut csv = String::from("timestamp,service,method,path,status\n");
            for log in logs {
                csv.push_str(&format!(
                    "{},{},{},{},{}\n",
                    log.timestamp.to_rfc3339(),
                    log.service,
                    log.method,
                    log.path,
                    log.status
                ));
            }

            Ok(axum::response::Response::builder()
                .header("Content-Type", "text/csv")
                .header("Content-Disposition", "attachment; filename=\"logs.csv\"")
                .body(csv.into())
                .unwrap())
        }
        _ => {
            // Default to JSON
            let json = serde_json::to_string_pretty(&logs).unwrap_or_default();

            Ok(axum::response::Response::builder()
                .header("Content-Type", "application/json")
                .header("Content-Disposition", "attachment; filename=\"logs.json\"")
                .body(json.into())
                .unwrap())
        }
    }
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

/// Generates a service definition using AI from a natural language prompt.
///
/// # Arguments
///
/// * `request` - The AI generation request containing the prompt.
#[axum::debug_handler]
pub async fn ai_generate(
    Json(request): Json<AiGenerateRequest>,
) -> Result<Json<ApiResponse<AiGenerateResponse>>, ApiError> {
    use crate::ai::{AiProvider, GeminiAiProvider, LocalAiProvider, OpenAiProvider};
    use crate::config::{load_config, AiProviderKind};
    use std::path::Path;

    // Validate prompt is not empty
    if request.prompt.trim().is_empty() {
        return Err(ApiError::bad_request(
            ApiErrorCode::InvalidParameter,
            "Prompt cannot be empty",
        ));
    }

    // Load configuration
    let cfg = match load_config(Path::new("apicentric.json")) {
        Ok(cfg) => cfg,
        Err(_) => {
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::ConfigLoadError,
                "Failed to load configuration file apicentric.json",
            ));
        }
    };
    let ai_cfg = match &cfg.ai {
        Some(cfg) => cfg,
        None => {
            return Err(ErrorResponse::ai_not_configured().into());
        }
    };

    // Determine which provider to use (from request or config)
    let provider_kind = if let Some(ref provider_str) = request.provider {
        match provider_str.to_lowercase().as_str() {
            "openai" => AiProviderKind::Openai,
            "gemini" => AiProviderKind::Gemini,
            "local" => AiProviderKind::Local,
            _ => {
                return Err(ApiError::bad_request(
                    ApiErrorCode::InvalidAiProvider,
                    format!(
                        "Unknown AI provider: {}. Use 'openai', 'gemini', or 'local'",
                        provider_str
                    ),
                ));
            }
        }
    } else {
        ai_cfg.provider.clone()
    };

    // Build provider based on configuration
    let provider: Box<dyn AiProvider> = match provider_kind {
        AiProviderKind::Local => {
            let path = ai_cfg
                .model_path
                .clone()
                .unwrap_or_else(|| "model.bin".to_string());
            Box::new(LocalAiProvider::new(path))
        }
        AiProviderKind::Openai => {
            let key = ai_cfg.api_key.clone().ok_or_else(|| {
                ApiError::bad_request(
                    ApiErrorCode::AiNotConfigured,
                    "OpenAI API key missing. Set ai.api_key in apicentric.json",
                )
            })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
            Box::new(OpenAiProvider::new(key, model))
        }
        AiProviderKind::Gemini => {
            let key = std::env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| ai_cfg.api_key.clone())
                .ok_or_else(|| {
                    ApiError::bad_request(
                        ApiErrorCode::AiNotConfigured,
                        "Gemini API key missing. Set GEMINI_API_KEY environment variable or ai.api_key in apicentric.json",
                    )
                })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
            Box::new(GeminiAiProvider::new(key, model))
        }
    };

    // Generate YAML from prompt
    let yaml = match provider.generate_yaml(&request.prompt).await {
        Ok(yaml) => yaml,
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::AiGenerationFailed,
                format!("AI generation failed: {}", e),
            ));
        }
    };

    // Validate the generated YAML
    let validation_errors = match serde_yaml::from_str::<ServiceDefinition>(&yaml) {
        Ok(def) => {
            // Validate the service definition
            match def.validate() {
                Ok(_) => Vec::new(),
                Err(errors) => errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect(),
            }
        }
        Err(e) => vec![format!("YAML parsing error: {}", e)],
    };

    let response = AiGenerateResponse {
        yaml,
        validation_errors,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Validates a YAML service definition.
///
/// # Arguments
///
/// * `request` - The validation request containing the YAML.
#[axum::debug_handler]
pub async fn ai_validate(
    Json(request): Json<AiValidateRequest>,
) -> Result<Json<ApiResponse<AiValidateResponse>>, StatusCode> {
    use crate::validation::ConfigValidator;

    let errors = match serde_yaml::from_str::<ServiceDefinition>(&request.yaml) {
        Ok(def) => {
            // Validate the service definition
            match def.validate() {
                Ok(_) => Vec::new(),
                Err(validation_errors) => validation_errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect(),
            }
        }
        Err(e) => vec![format!("YAML parsing error: {}", e)],
    };

    let response = AiValidateResponse {
        is_valid: errors.is_empty(),
        errors,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Checks the AI configuration status.
#[axum::debug_handler]
pub async fn ai_config_status() -> Result<Json<ApiResponse<AiConfigResponse>>, StatusCode> {
    use crate::config::{load_config, AiProviderKind};
    use std::path::Path;

    // Load configuration
    let cfg = match load_config(Path::new("apicentric.json")) {
        Ok(cfg) => cfg,
        Err(_) => {
            return Ok(Json(ApiResponse::error(
                "Failed to load configuration file apicentric.json".to_string(),
            )));
        }
    };
    let mut issues = Vec::new();

    let (is_configured, provider, model) = match &cfg.ai {
        Some(ai_cfg) => {
            let provider_str = match ai_cfg.provider {
                AiProviderKind::Openai => "openai",
                AiProviderKind::Gemini => "gemini",
                AiProviderKind::Local => "local",
            };

            // Check for provider-specific configuration issues
            match ai_cfg.provider {
                AiProviderKind::Openai => {
                    if ai_cfg.api_key.is_none() {
                        issues.push("OpenAI API key not configured".to_string());
                    }
                }
                AiProviderKind::Gemini => {
                    if ai_cfg.api_key.is_none() && std::env::var("GEMINI_API_KEY").is_err() {
                        issues.push(
                            "Gemini API key not configured (set GEMINI_API_KEY or ai.api_key)"
                                .to_string(),
                        );
                    }
                }
                AiProviderKind::Local => {
                    if let Some(ref path) = ai_cfg.model_path {
                        if !std::path::Path::new(path).exists() {
                            issues.push(format!("Local model file not found: {}", path));
                        }
                    } else {
                        issues.push("Local model path not configured".to_string());
                    }
                }
            }

            (
                issues.is_empty(),
                Some(provider_str.to_string()),
                ai_cfg.model.clone(),
            )
        }
        None => {
            issues.push("AI configuration not found in apicentric.json".to_string());
            (false, None, None)
        }
    };

    let response = AiConfigResponse {
        is_configured,
        provider,
        model,
        issues,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Starts a recording session.
///
/// # Arguments
///
/// * `request` - The request containing target URL and optional port.
/// * `recording_manager` - The recording session manager.
#[axum::debug_handler]
pub async fn start_recording(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
    Json(request): Json<StartRecordingRequest>,
) -> Result<Json<ApiResponse<StartRecordingResponse>>, StatusCode> {
    let port = request.port.unwrap_or(8888);

    match recording_manager
        .start_recording(request.target_url.clone(), port)
        .await
    {
        Ok((session_id, proxy_url, proxy_port)) => {
            let response = StartRecordingResponse {
                session_id,
                proxy_url,
                proxy_port,
                target_url: request.target_url,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

/// Stops the active recording session.
///
/// # Arguments
///
/// * `recording_manager` - The recording session manager.
#[axum::debug_handler]
pub async fn stop_recording(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
) -> Result<Json<ApiResponse<StopRecordingResponse>>, StatusCode> {
    match recording_manager.stop_recording().await {
        Ok((session_id, endpoints)) => {
            let response = StopRecordingResponse {
                session_id,
                captured_count: endpoints.len(),
                endpoints,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

/// Gets the status of the current recording session.
///
/// # Arguments
///
/// * `recording_manager` - The recording session manager.
#[axum::debug_handler]
pub async fn get_recording_status(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
) -> Result<Json<ApiResponse<RecordingStatusResponse>>, StatusCode> {
    let (is_active, session_id, proxy_url, proxy_port, target_url, captured_count) =
        recording_manager.get_status().await;

    let response = RecordingStatusResponse {
        is_active,
        session_id,
        proxy_url,
        proxy_port,
        target_url,
        captured_count,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Generates a service definition from the captured requests.
///
/// # Arguments
///
/// * `request` - The request containing service name and description.
/// * `recording_manager` - The recording session manager.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn generate_service_from_recording(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<GenerateServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Stop the recording and get the endpoints
    let (session_id, endpoints) = match recording_manager.stop_recording().await {
        Ok(result) => result,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    if endpoints.is_empty() {
        return Ok(Json(ApiResponse::error(
            "No requests were captured during recording".to_string(),
        )));
    }

    let endpoint_count = endpoints.len();

    // Create a service definition from the captured endpoints
    let service_def = ServiceDefinition {
        name: request.service_name.clone(),
        version: Some("1.0.0".to_string()),
        description: request.description.or(Some(format!(
            "Service generated from recording session {}",
            session_id
        ))),
        server: ServerConfig {
            port: None,
            base_path: "/".to_string(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        },
        models: None,
        fixtures: None,
        bucket: None,
        endpoints,
        graphql: None,
        behavior: None,
    };

    // Convert to YAML
    let yaml = match serde_yaml::to_string(&service_def) {
        Ok(y) => y,
        Err(e) => {
            return Ok(Json(ApiResponse::error(format!(
                "Failed to serialize service: {}",
                e
            ))))
        }
    };

    // Save the service file
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    let file_path =
        std::path::Path::new(&services_dir).join(format!("{}.yaml", request.service_name));

    // Check if file already exists
    if file_path.exists() {
        return Ok(Json(ApiResponse::error(format!(
            "Service file '{}' already exists",
            request.service_name
        ))));
    }

    // Create services directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&services_dir) {
        return Ok(Json(ApiResponse::error(format!(
            "Failed to create services directory: {}",
            e
        ))));
    }

    // Write the service definition to file
    match std::fs::write(&file_path, &yaml) {
        Ok(_) => {
            // Apply the service to the running simulator
            if let Err(e) = simulator.apply_service_definition(service_def).await {
                // Clean up the file if applying fails
                let _ = std::fs::remove_file(&file_path);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to apply service: {}",
                    e
                ))));
            }

            Ok(Json(ApiResponse::success(format!(
                "Service '{}' generated successfully from {} captured endpoints",
                request.service_name, endpoint_count
            ))))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to write service file: {}",
            e
        )))),
    }
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

/// Generates TypeScript types for a service definition.
///
/// # Arguments
///
/// * `request` - The request containing the service name.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn generate_typescript(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<TypeScriptGenerateRequest>,
) -> Result<Json<ApiResponse<TypeScriptGenerateResponse>>, ApiError> {
    use crate::simulator::typescript::to_typescript;

    // Validate service name
    validation::validate_service_name(&request.service_name).map_err(ApiError::from)?;

    // Get the service definition
    let registry = simulator.service_registry().read().await;

    let service_arc = match registry.get_service(&request.service_name) {
        Some(s) => s,
        None => {
            return Err(ErrorResponse::service_not_found(&request.service_name).into());
        }
    };

    let service = service_arc.read().await;
    let definition = service.definition();

    // Generate TypeScript types
    match to_typescript(&definition) {
        Ok(code) => {
            let response = TypeScriptGenerateResponse { code };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::CodeGenerationFailed,
            format!("Failed to generate TypeScript types: {}", e),
        )),
    }
}

/// Generates React Query hooks for a service definition.
///
/// # Arguments
///
/// * `request` - The request containing the service name.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn generate_react_query(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<ReactQueryGenerateRequest>,
) -> Result<Json<ApiResponse<ReactQueryGenerateResponse>>, ApiError> {
    use crate::simulator::react_query::to_react_query;

    // Validate service name
    validation::validate_service_name(&request.service_name).map_err(ApiError::from)?;

    // Get the service definition
    let registry = simulator.service_registry().read().await;

    let service_arc = match registry.get_service(&request.service_name) {
        Some(s) => s,
        None => {
            return Err(ErrorResponse::service_not_found(&request.service_name).into());
        }
    };

    let service = service_arc.read().await;
    let definition = service.definition();

    // Generate React Query hooks
    match to_react_query(&definition) {
        Ok(code) => {
            let response = ReactQueryGenerateResponse { code };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::CodeGenerationFailed,
            format!("Failed to generate React Query hooks: {}", e),
        )),
    }
}

/// Generates an Axios client for a service definition.
///
/// # Arguments
///
/// * `request` - The request containing the service name.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn generate_axios(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<AxiosGenerateRequest>,
) -> Result<Json<ApiResponse<AxiosGenerateResponse>>, ApiError> {
    use crate::simulator::axios_client::to_axios_client;

    // Validate service name
    validation::validate_service_name(&request.service_name).map_err(ApiError::from)?;

    // Get the service definition
    let registry = simulator.service_registry().read().await;

    let service_arc = match registry.get_service(&request.service_name) {
        Some(s) => s,
        None => {
            return Err(ErrorResponse::service_not_found(&request.service_name).into());
        }
    };

    let service = service_arc.read().await;
    let definition = service.definition();

    // Generate Axios client
    match to_axios_client(&definition) {
        Ok(code) => {
            let response = AxiosGenerateResponse { code };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::CodeGenerationFailed,
            format!("Failed to generate Axios client: {}", e),
        )),
    }
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

/// Gets the current Apicentric configuration.
#[axum::debug_handler]
pub async fn get_config() -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    use crate::config::load_config;
    use std::path::Path;

    let config_path =
        std::env::var("APICENTRIC_CONFIG_PATH").unwrap_or_else(|_| "apicentric.json".to_string());

    match load_config(Path::new(&config_path)) {
        Ok(config) => {
            // Convert to JSON for easier manipulation in the frontend
            match serde_json::to_value(&config) {
                Ok(json) => Ok(Json(ApiResponse::success(json))),
                Err(e) => Ok(Json(ApiResponse::error(format!(
                    "Failed to serialize configuration: {}",
                    e
                )))),
            }
        }
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to load configuration: {}",
            e
        )))),
    }
}

/// Updates the Apicentric configuration.
///
/// # Arguments
///
/// * `request` - The request containing the updated configuration.
#[axum::debug_handler]
pub async fn update_config(
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    use crate::config::{save_config, ApicentricConfig};
    use crate::validation::ConfigValidator;
    use std::path::Path;

    let config_path =
        std::env::var("APICENTRIC_CONFIG_PATH").unwrap_or_else(|_| "apicentric.json".to_string());

    // Parse the JSON into ApicentricConfig
    let config: ApicentricConfig = match serde_json::from_value(request.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            return Ok(Json(ApiResponse::error(format!(
                "Invalid configuration format: {}",
                e
            ))));
        }
    };

    // Validate the configuration
    if let Err(validation_errors) = config.validate() {
        let error_messages: Vec<String> = validation_errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        return Ok(Json(ApiResponse::error(format!(
            "Configuration validation failed:\n{}",
            error_messages.join("\n")
        ))));
    }

    // Save the configuration
    match save_config(&config, Path::new(&config_path)) {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Configuration updated successfully".to_string(),
        ))),
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to save configuration: {}",
            e
        )))),
    }
}

/// Validates a configuration without saving it.
///
/// # Arguments
///
/// * `request` - The request containing the configuration to validate.
#[axum::debug_handler]
pub async fn validate_config(
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<ApiResponse<ValidateConfigResponse>>, StatusCode> {
    use crate::config::ApicentricConfig;
    use crate::validation::ConfigValidator;

    // Parse the JSON into ApicentricConfig
    let config: ApicentricConfig = match serde_json::from_value(request.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            let response = ValidateConfigResponse {
                is_valid: false,
                errors: vec![format!("Invalid configuration format: {}", e)],
            };
            return Ok(Json(ApiResponse::success(response)));
        }
    };

    // Validate the configuration
    let errors = match config.validate() {
        Ok(_) => Vec::new(),
        Err(validation_errors) => validation_errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect(),
    };

    let response = ValidateConfigResponse {
        is_valid: errors.is_empty(),
        errors,
    };

    Ok(Json(ApiResponse::success(response)))
}

// ============================================
// Monitoring and Metrics Endpoints
// ============================================

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

/// Gets application metrics.
///
/// # Arguments
///
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn get_metrics(
    State(_simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<MetricsResponse>>, StatusCode> {
    // TODO: Implement proper metrics collection
    // Get service count
    // let services = simulator.list_services().await;
    let active_services = 0; // services.iter().filter(|s| s.is_running).count() as u64;

    // Get log count
    // let logs = simulator.get_logs().await;
    let total_log_entries = 0; // logs.len() as u64;

    // Create metrics response
    let metrics = MetricsResponse {
        total_requests: 0, // Would need to track this in middleware
        successful_requests: 0,
        failed_requests: 0,
        avg_response_time_ms: 0.0,
        active_websocket_connections: 0, // Would need to track this in WebSocket handler
        active_services,
        total_log_entries,
        uptime_seconds: 0, // Would need to track server start time
        memory_usage_bytes: get_memory_usage(),
    };

    Ok(Json(ApiResponse::success(metrics)))
}

// ============================================
// Marketplace and Import Endpoints
// ============================================

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

use crate::simulator::marketplace::{get_marketplace_items, MarketplaceItem};

/// Import a service definition from a URL.
///
/// # Arguments
///
/// * `request` - The request containing the URL.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn import_from_url(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<ImportUrlRequest>,
) -> Result<Json<ApiResponse<ImportUrlResponse>>, ApiError> {
    use crate::simulator::openapi::from_openapi;

    // Fetch the content from URL
    let content = match reqwest::get(&request.url).await {
        Ok(res) => match res.text().await {
            Ok(text) => text,
            Err(e) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    ApiErrorCode::ImportFailed,
                    format!("Failed to read content from URL: {}", e),
                ));
            }
        },
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::ImportFailed,
                format!("Failed to fetch URL: {}", e),
            ));
        }
    };

    // Try to parse as YAML/JSON
    let value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::InvalidYaml,
                format!("Failed to parse content as YAML/JSON: {}", e),
            ));
        }
    };

    // Determine if it's OpenAPI
    let definition = if value.get("openapi").is_some() || value.get("swagger").is_some() {
        // Convert OpenAPI to ServiceDefinition
        from_openapi(&value)
    } else {
        // Try to parse as ServiceDefinition directly
        match serde_yaml::from_value::<ServiceDefinition>(value) {
            Ok(def) => def,
            Err(_) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    ApiErrorCode::InvalidServiceDefinition,
                    "Content is neither a valid OpenAPI spec nor an Apicentric ServiceDefinition"
                        .to_string(),
                ));
            }
        }
    };

    // Validate service name
    validation::validate_service_name(&definition.name).map_err(ApiError::from)?;

    // Generate YAML
    let yaml = match serde_yaml::to_string(&definition) {
        Ok(y) => y,
        Err(e) => {
            return Err(ApiError::internal_server_error(format!(
                "Failed to serialize service: {}",
                e
            )));
        }
    };

    // Save to file
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    let file_path = std::path::Path::new(&services_dir).join(format!("{}.yaml", definition.name));

    // Create services directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&services_dir) {
        return Err(ApiError::internal_server_error(format!(
            "Failed to create services directory: {}",
            e
        )));
    }

    match std::fs::write(&file_path, &yaml) {
        Ok(_) => {
            // Apply the service to the running simulator
            if let Err(e) = simulator.apply_service_definition(definition.clone()).await {
                // Clean up the file if applying fails
                let _ = std::fs::remove_file(&file_path);
                return Err(ApiError::internal_server_error(format!(
                    "Failed to apply service: {}",
                    e
                )));
            }

            let response = ImportUrlResponse {
                service_name: definition.name,
                yaml,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Err(ApiError::internal_server_error(format!(
            "Failed to write service file: {}",
            e
        ))),
    }
}

/// Lists available marketplace items.
#[axum::debug_handler]
pub async fn marketplace_list() -> Result<Json<ApiResponse<Vec<MarketplaceItem>>>, StatusCode> {
    let items = get_marketplace_items();
    Ok(Json(ApiResponse::success(items)))
}

/// Gets current memory usage (Linux only)
fn get_memory_usage() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb_val) = kb.parse::<u64>() {
                            return Some(kb_val * 1024);
                        }
                    }
                    break;
                }
            }
        }
    }
    None
}

// ============================================================================
// Legacy Simulator Status Endpoints (for backward compatibility with old frontend)
// ============================================================================

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

/// Gets the simulator status (legacy endpoint for old frontend)
///
/// GET /status
#[axum::debug_handler]
pub async fn get_simulator_status(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<LegacySimulatorStatus>>, StatusCode> {
    // Get simulator status
    let status = simulator.get_status().await;

    let active_services: Vec<LegacyServiceInfo> = status
        .active_services
        .into_iter()
        .map(|s| LegacyServiceInfo {
            name: s.name,
            port: s.port,
            is_running: s.is_running,
            endpoints_count: s.endpoints_count,
        })
        .collect();

    Ok(Json(ApiResponse::success(LegacySimulatorStatus {
        active_services,
        is_running: status.is_active,
    })))
}

/// Starts the simulator (legacy endpoint)
///
/// POST /start
#[axum::debug_handler]
pub async fn start_simulator(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match simulator.start().await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Simulator started successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to start simulator: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Stops the simulator (legacy endpoint)
///
/// POST /stop
#[axum::debug_handler]
pub async fn stop_simulator(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match simulator.stop().await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Simulator stopped successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to stop simulator: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
