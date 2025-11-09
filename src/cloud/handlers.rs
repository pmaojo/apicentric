//! Axum handlers for the cloud API.
//!
//! This module provides handlers for listing, loading, and saving services.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::simulator::{ApiSimulatorManager, ServiceDefinition, ServiceInfo};
use crate::simulator::log::RequestLogEntry;

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

/// A query for logs.
#[derive(Deserialize)]
pub struct LogsQuery {
    /// The maximum number of logs to return.
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
