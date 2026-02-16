use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::error::{validation, ApiError, ApiErrorCode};
use crate::cloud::types::ApiResponse;
use crate::simulator::marketplace::{get_marketplace_items, MarketplaceItem};
use crate::simulator::{ApiSimulatorManager, ServiceDefinition, UnifiedConfig};
use crate::utils::validate_ssrf_url;
use reqwest::redirect::Policy;

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

    // Validate SSRF
    let (url, socket_addr) = validate_ssrf_url(&request.url)
        .await
        .map_err(|e| ApiError::bad_request(ApiErrorCode::InvalidParameter, e))?;

    // Create a client that resolves the host to the validated IP and disables redirects
    // to prevent redirection to internal services after initial check.
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .resolve(url.host_str().unwrap(), socket_addr)
        .build()
        .map_err(|e| {
            ApiError::internal_server_error(format!("Failed to build HTTP client: {}", e))
        })?;

    // Fetch the content from URL
    let res = match client.get(url).send().await {
        Ok(res) => res,
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::ImportFailed,
                format!("Failed to fetch URL: {}", e),
            ));
        }
    };

    if !res.status().is_success() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::ImportFailed,
            format!(
                "Remote server returned error: {} for URL: {}",
                res.status(),
                request.url
            ),
        ));
    }

    let content = match res.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::ImportFailed,
                format!("Failed to read content from URL: {}", e),
            ));
        }
    };

    // Try to parse as YAML/JSON
    let value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::InvalidServiceDefinition,
                format!("Failed to parse content as YAML/JSON: {}", e),
            ));
        }
    };

    // Determine if it's OpenAPI
    let is_openapi = value.get("openapi").is_some() || value.get("swagger").is_some();

    let mut definition = if is_openapi {
        // Convert OpenAPI to ServiceDefinition
        match from_openapi(&value) {
            Ok(def) => def,
            Err(e) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    ApiErrorCode::InvalidServiceDefinition,
                    format!("Failed to parse OpenAPI/Swagger spec: {}", e),
                ));
            }
        }
    } else {
        // Try to parse (using UnifiedConfig for Digital Twin support)
        match serde_yaml::from_value::<UnifiedConfig>(value) {
            Ok(unified) => ServiceDefinition::from(unified),
            Err(e) => {
                return Err(ApiError::new(
                    StatusCode::BAD_REQUEST,
                    ApiErrorCode::InvalidServiceDefinition,
                    format!("Content is neither a valid OpenAPI spec, Apicentric ServiceDefinition, nor Digital Twin. Parse error: {}", e),
                ));
            }
        }
    };

    // Sanitize service name to ensure it passes validation
    let sanitized_name = definition
        .name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        // Reduce multiple hyphens to single hyphen
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if !sanitized_name.is_empty() {
        definition.name = sanitized_name;
    }

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

    // Resolve safe path using simulator manager
    let file_path = simulator
        .resolve_service_path(&format!("{}.yaml", definition.name))
        .map_err(|e| ApiError::internal_server_error(format!("Invalid file path: {}", e)))?;

    match simulator.save_service_file(&file_path, &yaml) {
        Ok(_) => {
            // Apply the service to the running simulator
            if let Err(e) = simulator.apply_service_definition(definition.clone()).await {
                let error_msg = e.to_string();
                if error_msg.contains("already registered") {
                    return Err(ApiError::new(
                        StatusCode::CONFLICT,
                        ApiErrorCode::ServiceAlreadyExists,
                        format!("Failed to apply service: {}", error_msg),
                    ));
                }
                return Err(ApiError::internal_server_error(format!(
                    "Failed to apply service: {}",
                    error_msg
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
