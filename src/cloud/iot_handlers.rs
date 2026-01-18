use axum::{
    extract::{Path, Multipart},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::cloud::error::{ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::handlers::ApiResponse;
use crate::iot::config::TwinConfig;

/// Lists all available twins.
pub async fn list_twins() -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let path = std::path::Path::new(&iot_dir);

    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| {
            ApiError::internal_server_error(format!("Failed to create IoT directory: {}", e))
        })?;
    }

    let mut twins = Vec::new();
    let entries = std::fs::read_dir(path).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to read IoT directory: {}", e))
    })?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(name) = stem.to_str() {
                            twins.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(Json(ApiResponse::success(twins)))
}

#[derive(Serialize)]
pub struct TwinDetailResponse {
    pub name: String,
    pub yaml: String,
    pub config: TwinConfig,
}

/// Gets a specific twin definition.
pub async fn get_twin(
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<TwinDetailResponse>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let path = std::path::Path::new(&iot_dir).join(format!("{}.yaml", name));

    if !path.exists() {
        return Err(ErrorResponse::service_not_found(&name).into()); // Reuse service not found or generic not found
    }

    let content = std::fs::read_to_string(&path).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to read twin file: {}", e))
    })?;

    let config: TwinConfig = serde_yaml::from_str(&content).map_err(|e| {
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, ApiErrorCode::InvalidYaml, format!("Invalid YAML: {}", e))
    })?;

    Ok(Json(ApiResponse::success(TwinDetailResponse {
        name,
        yaml: content,
        config,
    })))
}

#[derive(Deserialize)]
pub struct SaveTwinRequest {
    pub yaml: String,
}

/// Creates or updates a twin.
pub async fn save_twin(
    Path(name): Path<String>,
    Json(request): Json<SaveTwinRequest>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let dir_path = std::path::Path::new(&iot_dir);

    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path).map_err(|e| {
            ApiError::internal_server_error(format!("Failed to create IoT directory: {}", e))
        })?;
    }

    let path = dir_path.join(format!("{}.yaml", name));

    // Validate YAML
    let _: TwinConfig = serde_yaml::from_str(&request.yaml).map_err(|e| {
        ApiError::new(StatusCode::BAD_REQUEST, ApiErrorCode::InvalidYaml, format!("Invalid YAML: {}", e))
    })?;

    std::fs::write(&path, &request.yaml).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to write twin file: {}", e))
    })?;

    Ok(Json(ApiResponse::success(format!("Twin '{}' saved", name))))
}

/// Deletes a twin.
pub async fn delete_twin(
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let path = std::path::Path::new(&iot_dir).join(format!("{}.yaml", name));

    if !path.exists() {
        return Err(ErrorResponse::service_not_found(&name).into());
    }

    std::fs::remove_file(&path).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to delete twin file: {}", e))
    })?;

    Ok(Json(ApiResponse::success(format!("Twin '{}' deleted", name))))
}

/// Uploads a CSV file for replay strategy.
pub async fn upload_replay_data(
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let dir_path = std::path::Path::new(&iot_dir);

    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path).map_err(|e| {
            ApiError::internal_server_error(format!("Failed to create IoT directory: {}", e))
        })?;
    }

    let mut filename = String::new();
    let mut saved_path = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::bad_request(ApiErrorCode::InvalidParameter, format!("Multipart error: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let raw_file_name = field.file_name().unwrap_or("data.csv");

            // Sanitize filename to prevent directory traversal
            let file_name = std::path::Path::new(raw_file_name)
                .file_name()
                .ok_or_else(|| ApiError::bad_request(ApiErrorCode::InvalidParameter, "Invalid filename".to_string()))?
                .to_string_lossy()
                .to_string();

            filename = file_name.clone();
            let data = field.bytes().await.map_err(|e| {
                ApiError::internal_server_error(format!("Failed to read file bytes: {}", e))
            })?;

            let path = dir_path.join(&file_name);
            std::fs::write(&path, data).map_err(|e| {
                ApiError::internal_server_error(format!("Failed to write file: {}", e))
            })?;
            saved_path = path.to_string_lossy().to_string();
        }
    }

    if saved_path.is_empty() {
         return Err(ApiError::bad_request(ApiErrorCode::InvalidParameter, "No file uploaded".to_string()));
    }

    Ok(Json(ApiResponse::success(filename)))
}
