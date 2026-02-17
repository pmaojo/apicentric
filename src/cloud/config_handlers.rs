//! Axum handlers for configuration management.

use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

use crate::cloud::types::ApiResponse;

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
