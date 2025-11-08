use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::simulator::{ApiSimulatorManager, ServiceDefinition, ServiceInfo};
use crate::simulator::log::RequestLogEntry;

// Response DTOs
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Request DTOs
#[derive(Deserialize)]
pub struct LoadServiceRequest {
    pub path: String,
}

#[derive(Deserialize)]
pub struct SaveServiceRequest {
    pub path: String,
    pub yaml: String,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub limit: Option<usize>,
}

// Handlers
#[axum::debug_handler]
pub async fn list_services(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<Vec<ServiceInfo>>>, StatusCode> {
    let status = simulator.get_status().await;
    Ok(Json(ApiResponse::success(status.active_services)))
}

#[axum::debug_handler]
pub async fn load_service(
    Json(request): Json<LoadServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match std::fs::File::open(&request.path) {
        Ok(file) => {
            match serde_yaml::from_reader::<_, ServiceDefinition>(file) {
                Ok(def) => {
                    match serde_yaml::to_string(&def) {
                        Ok(yaml) => Ok(Json(ApiResponse::success(yaml))),
                        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
                    }
                },
                Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
            }
        },
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

#[axum::debug_handler]
pub async fn save_service(
    Json(request): Json<SaveServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match serde_yaml::from_str::<ServiceDefinition>(&request.yaml) {
        Ok(def) => {
            match std::fs::File::create(&request.path) {
                Ok(file) => {
                    match serde_yaml::to_writer(file, &def) {
                        Ok(_) => Ok(Json(ApiResponse::success("Service saved".to_string()))),
                        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
                    }
                },
                Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
            }
        },
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}