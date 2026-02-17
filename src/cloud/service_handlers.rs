//! Axum handlers for service management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::error::{validation, ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::types::ApiResponse;
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
/// * `simulator` - The API simulator manager.
/// * `request` - The request to load the service.
#[axum::debug_handler]
pub async fn load_service(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<LoadServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let safe_path = match simulator.resolve_service_path(&request.path) {
        Ok(path) => path,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
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
/// * `simulator` - The API simulator manager.
/// * `request` - The request to save the service.
#[axum::debug_handler]
pub async fn save_service(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<SaveServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let safe_path = match simulator.resolve_service_path(&request.path) {
        Ok(path) => path,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    match serde_yaml::from_str::<UnifiedConfig>(&request.yaml) {
        Ok(unified) => {
            let def = ServiceDefinition::from(unified);
            // Re-serialize to ensure valid YAML structure
            match serde_yaml::to_string(&def) {
                Ok(yaml_content) => match simulator.save_service_file(&safe_path, &yaml_content) {
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

    // Sentinel: Use resolve_safe_service_path to prevent directory traversal
    let file_path = match simulator.resolve_service_path(&filename) {
        Ok(path) => path,
        Err(e) => {
            return Err(ApiError::bad_request(
                ApiErrorCode::InvalidParameter,
                e.to_string(),
            ))
        }
    };

    // Check if file already exists
    if simulator.service_file_exists(&file_path) {
        return Err(ErrorResponse::service_already_exists(&filename).into());
    }

    // Write the service definition to file
    match simulator.save_service_file(&file_path, &request.yaml) {
        Ok(_) => {
            // Apply the service to the running simulator
            if let Err(e) = simulator.apply_service_definition(definition).await {
                // Clean up the file if applying fails
                let _ = simulator.delete_service_file(&file_path);
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

    let schema_filename = format!("{}_schema.graphql", request.name);
    let mock_filename = format!("{}_mock.json", request.name);
    let service_filename = format!("{}.yaml", request.name);

    let schema_path = simulator
        .resolve_service_path(&schema_filename)
        .map_err(|e| ApiError::internal_server_error(format!("Invalid path: {}", e)))?;
    let mock_path = simulator
        .resolve_service_path(&mock_filename)
        .map_err(|e| ApiError::internal_server_error(format!("Invalid path: {}", e)))?;
    let service_path = simulator
        .resolve_service_path(&service_filename)
        .map_err(|e| ApiError::internal_server_error(format!("Invalid path: {}", e)))?;

    // Create YAML content (using absolute paths for schema/mocks as required by simulator config)
    // Note: simulator configuration usually expects paths relative to CWD or absolute.
    // Here we use the resolved absolute paths.
    let yaml_content = format!(
        r#"name: {}
version: 1.0.0
description: A GraphQL service generated by Apicentric.
server:
  port: {}
  base_path: /graphql
graphql:
  schema_path: {}
  mocks:
    helloQuery: {}"#,
        request.name,
        request.port,
        schema_path.display(),
        mock_path.display()
    );

    simulator
        .save_service_file(&schema_path, "type Query {\n  hello: String\n}")
        .map_err(|e| ApiError::internal_server_error(format!("Failed to write schema: {}", e)))?;

    simulator
        .save_service_file(
            &mock_path,
            "{\n  \"data\": {\n    \"hello\": \"world\"\n  }\n}",
        )
        .map_err(|e| ApiError::internal_server_error(format!("Failed to write mock: {}", e)))?;

    simulator
        .save_service_file(&service_path, &yaml_content)
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
    let file_path = match simulator.resolve_service_path(&format!("{}.yaml", name)) {
        Ok(path) => path,
        Err(e) => {
            return Err(ApiError::internal_server_error(format!(
                "Invalid path: {}",
                e
            )))
        }
    };

    if !simulator.service_file_exists(&file_path) {
        return Err(ErrorResponse::service_not_found(&name).into());
    }

    // Write the updated definition
    match simulator.save_service_file(&file_path, &request.yaml) {
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
    let file_path = match simulator.resolve_service_path(&format!("{}.yaml", name)) {
        Ok(path) => path,
        Err(e) => {
            return Err(ApiError::internal_server_error(format!(
                "Invalid path: {}",
                e
            )))
        }
    };

    if !simulator.service_file_exists(&file_path) {
        return Err(ErrorResponse::service_not_found(&name).into());
    }

    match simulator.delete_service_file(&file_path) {
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
