use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::error::{validation, ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::types::ApiResponse;
use crate::simulator::ApiSimulatorManager;

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
    use crate::simulator::react_query::generate_react_query_hooks;

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
    match generate_react_query_hooks(&definition) {
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
    use crate::simulator::axios_client::generate_axios_client;

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
    match generate_axios_client(&definition) {
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
