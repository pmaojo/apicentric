//! Axum handlers for legacy/stub endpoints.

use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;
use std::sync::Arc;

use crate::cloud::types::ApiResponse;
use crate::simulator::ApiSimulatorManager;

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
