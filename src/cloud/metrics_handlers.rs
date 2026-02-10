use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;
use crate::cloud::types::ApiResponse;
use crate::simulator::ApiSimulatorManager;

/// Response for metrics endpoint
#[derive(Serialize)]
pub struct MetricsResponse {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Number of active WebSocket connections
    pub active_websocket_connections: u64,
    /// Number of active services
    pub active_services: u64,
    /// Total number of log entries
    pub total_log_entries: u64,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<u64>,
}

/// Gets application metrics.
///
/// # Arguments
///
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn get_metrics(
    State(simulator): State<Arc<ApiSimulatorManager>>,
) -> Result<Json<ApiResponse<MetricsResponse>>, StatusCode> {
    // Get service count
    let registry = simulator.service_registry().read().await;
    let active_services = registry.running_services_count().await as u64;

    // Get log stats
    let storage = registry.storage();
    let stats = storage.get_log_stats().unwrap_or_default();

    // Get uptime
    let uptime_seconds = simulator.get_uptime_seconds();

    // Create metrics response
    let metrics = MetricsResponse {
        total_requests: stats.total_requests,
        successful_requests: stats.successful_requests,
        failed_requests: stats.failed_requests,
        avg_response_time_ms: stats.avg_response_time_ms,
        active_websocket_connections: 0, // Would need to track this in WebSocket handler
        active_services,
        total_log_entries: stats.total_requests,
        uptime_seconds,
        memory_usage_bytes: get_memory_usage(),
    };

    Ok(Json(ApiResponse::success(metrics)))
}

/// Gets current memory usage (Linux only)
fn get_memory_usage() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb_val) = kb.parse::<u64>() {
                            return Some(kb_val * 1024);
                        }
                    }
                    break;
                }
            }
        }
    }
    None
}
