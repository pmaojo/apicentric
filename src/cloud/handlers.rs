//! Axum handlers for the cloud API.
//!
//! This module provides handlers for listing, loading, and saving services.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::error::{validation, ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::fs_utils::resolve_safe_service_path;
use crate::cloud::types::ApiResponse;
use crate::simulator::log::RequestLogEntry;
use crate::simulator::{ApiSimulatorManager, ServiceDefinition, ServiceInfo, UnifiedConfig};

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
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());

    let safe_path = match resolve_safe_service_path(&services_dir, &request.path) {
        Ok(path) => path,
        Err(e) => return Ok(Json(ApiResponse::error(e))),
    };

    match std::fs::read_to_string(&safe_path) {
        Ok(content) => match serde_yaml::from_str::<UnifiedConfig>(&content) {
            Ok(unified) => {
                let def = ServiceDefinition::from(unified);
                match serde_yaml::to_string(&def) {
                    Ok(yaml) => Ok(Json(ApiResponse::success(yaml))),
                    Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
                }
            }
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
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());

    let safe_path = match resolve_safe_service_path(&services_dir, &request.path) {
        Ok(path) => path,
        Err(e) => return Ok(Json(ApiResponse::error(e))),
    };

    match serde_yaml::from_str::<UnifiedConfig>(&request.yaml) {
        Ok(unified) => {
            let def = ServiceDefinition::from(unified);
            match std::fs::File::create(&safe_path) {
                Ok(file) => match serde_yaml::to_writer(file, &def) {
                    Ok(_) => Ok(Json(ApiResponse::success("Service saved".to_string()))),
                    Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
                },
                Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
            }
        }
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
            endpoint_count: service
                .definition()
                .endpoints
                .as_ref()
                .map(|e| e.len())
                .unwrap_or(0),
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
                    base_path: definition
                        .server
                        .as_ref()
                        .map(|s| s.base_path.clone())
                        .unwrap_or_else(|| "/".to_string()),
                    endpoints_count: definition.endpoints.as_ref().map(|e| e.len()).unwrap_or(0),
                    is_running: service.is_running(),
                    version: definition
                        .version
                        .clone()
                        .unwrap_or_else(|| "1.0.0".to_string()),
                    definition: yaml.clone(),
                    endpoints: definition.endpoints.clone().unwrap_or_default(),
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

/// Gets the OpenAPI specification for a specific service.
///
/// # Arguments
///
/// * `name` - The name of the service.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn get_service_openapi(
    Path(name): Path<String>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&name).map_err(ApiError::from)?;

    let registry = simulator.service_registry().read().await;

    if let Some(service_arc) = registry.get_service(&name) {
        let service = service_arc.read().await;
        let definition = service.definition();

        // Convert to OpenAPI
        let openapi = crate::simulator::openapi::to_openapi(&definition);

        // Serialize to JSON Value
        match serde_json::to_value(&openapi) {
            Ok(json) => Ok(Json(ApiResponse::success(json))),
            Err(e) => Err(ApiError::internal_server_error(format!(
                "Failed to serialize OpenAPI spec: {}",
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

    // Parse the YAML (using UnifiedConfig for Digital Twin support)
    let definition: ServiceDefinition = match serde_yaml::from_str::<UnifiedConfig>(&request.yaml) {
        Ok(unified) => ServiceDefinition::from(unified),
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

    // Sentinel: Use resolve_safe_service_path to prevent directory traversal
    let file_path = match resolve_safe_service_path(&services_dir, &filename) {
        Ok(path) => path,
        Err(e) => return Err(ApiError::bad_request(ApiErrorCode::InvalidParameter, e)),
    };

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

/// Creates a new GraphQL service.
#[axum::debug_handler]
pub async fn create_graphql_service(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<CreateGraphQLServiceRequest>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Validate service name
    validation::validate_service_name(&request.name).map_err(ApiError::from)?;

    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    let schema_filename = format!("{}_schema.graphql", request.name);
    let mock_filename = format!("{}_mock.json", request.name);
    let service_filename = format!("{}.yaml", request.name);

    // Create YAML content
    let yaml_content = format!(
        r#"name: {}
version: 1.0.0
description: A GraphQL service generated by Apicentric.
server:
  port: {}
  base_path: /graphql
graphql:
  schema_path: {}/{}
  mocks:
    helloQuery: {}/{}"#,
        request.name, request.port, services_dir, schema_filename, services_dir, mock_filename
    );

    // Create files... (omitting actual file creation logic for brevity in replacement, but using std::fs)
    let base_path = std::path::Path::new(&services_dir);
    if let Err(e) = std::fs::create_dir_all(base_path) {
        return Err(ApiError::internal_server_error(format!(
            "Failed to create directory: {}",
            e
        )));
    }

    std::fs::write(
        base_path.join(&schema_filename),
        "type Query {\n  hello: String\n}",
    )
    .map_err(|e| ApiError::internal_server_error(format!("Failed to write schema: {}", e)))?;

    std::fs::write(
        base_path.join(&mock_filename),
        "{\n  \"data\": {\n    \"hello\": \"world\"\n  }\n}",
    )
    .map_err(|e| ApiError::internal_server_error(format!("Failed to write mock: {}", e)))?;

    std::fs::write(base_path.join(&service_filename), &yaml_content)
        .map_err(|e| ApiError::internal_server_error(format!("Failed to write service: {}", e)))?;

    // Apply the service
    let definition: ServiceDefinition = serde_yaml::from_str(&yaml_content).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to parse generated YAML: {}", e))
    })?;

    if let Err(e) = simulator.apply_service_definition(definition).await {
        return Err(if e.to_string().contains("already registered") {
            ApiError::conflict(
                crate::cloud::error::ApiErrorCode::ServiceAlreadyExists,
                format!("Service '{}' is already registered", request.name),
            )
        } else {
            ApiError::internal_server_error(format!("Failed to apply service: {}", e))
        });
    }

    Ok(Json(ApiResponse::success(format!(
        "GraphQL service '{}' created",
        request.name
    ))))
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

/// Validates a simulator configuration.
#[axum::debug_handler]
pub async fn validate_simulator(
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Basic validation stub for now - alignment with frontend /api/simulator/validate
    Ok(Json(ApiResponse::success(serde_json::json!({
        "valid": true,
        "message": "Configuration is valid"
    }))))
}

/// Runs contract tests for a service.
#[axum::debug_handler]
pub async fn run_contract_tests(
    Json(_service): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Contract testing stub for now - alignment with frontend /api/contract-testing
    Ok(Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "results": [],
        "summary": {
            "passed": 0,
            "failed": 0,
            "total": 0
        }
    }))))
}
