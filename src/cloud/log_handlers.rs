//! Axum handlers for logging.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::cloud::types::ApiResponse;
use crate::simulator::log::RequestLogEntry;
use crate::simulator::ApiSimulatorManager;

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
                    crate::utils::sanitize_csv_field(&log.service),
                    crate::utils::sanitize_csv_field(&log.method),
                    crate::utils::sanitize_csv_field(&log.path),
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
