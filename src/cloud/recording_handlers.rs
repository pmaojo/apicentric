use axum::{extract::State, http::StatusCode, response::Json, Extension};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cloud::error::validation;
use crate::cloud::recording_session::RecordingSessionManager;
use crate::cloud::types::ApiResponse;
use crate::simulator::config::{EndpointDefinition, ServerConfig};
use crate::simulator::{ApiSimulatorManager, ServiceDefinition};

/// Request to start recording mode.
#[derive(Deserialize)]
pub struct StartRecordingRequest {
    /// The target URL to proxy to.
    pub target_url: String,
    /// The port to listen on (optional, defaults to 8888).
    pub port: Option<u16>,
}

/// Response for starting recording mode.
#[derive(Serialize)]
pub struct StartRecordingResponse {
    /// The session ID for this recording.
    pub session_id: String,
    /// The proxy URL to use.
    pub proxy_url: String,
    /// The port the proxy is listening on.
    pub proxy_port: u16,
    /// The target URL being proxied.
    pub target_url: String,
}

/// Response for recording status.
#[derive(Serialize)]
pub struct RecordingStatusResponse {
    /// Whether recording is currently active.
    pub is_active: bool,
    /// The session ID if recording is active.
    pub session_id: Option<String>,
    /// The proxy URL if recording is active.
    pub proxy_url: Option<String>,
    /// The proxy port if recording is active.
    pub proxy_port: Option<u16>,
    /// The target URL if recording is active.
    pub target_url: Option<String>,
    /// Number of requests captured so far.
    pub captured_count: usize,
}

/// Response for stopping recording mode.
#[derive(Serialize)]
pub struct StopRecordingResponse {
    /// The session ID that was stopped.
    pub session_id: String,
    /// Number of requests captured.
    pub captured_count: usize,
    /// The captured endpoints.
    pub endpoints: Vec<EndpointDefinition>,
}

/// Request to generate a service from captured requests.
#[derive(Deserialize)]
pub struct GenerateServiceRequest {
    /// The name for the generated service.
    pub service_name: String,
    /// Optional description for the service.
    pub description: Option<String>,
}

/// Starts a recording session.
///
/// # Arguments
///
/// * `request` - The request containing target URL and optional port.
/// * `recording_manager` - The recording session manager.
#[axum::debug_handler]
pub async fn start_recording(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
    Json(request): Json<StartRecordingRequest>,
) -> Result<Json<ApiResponse<StartRecordingResponse>>, StatusCode> {
    let port = request.port.unwrap_or(8888);

    match recording_manager
        .start_recording(request.target_url.clone(), port)
        .await
    {
        Ok((session_id, proxy_url, proxy_port)) => {
            let response = StartRecordingResponse {
                session_id,
                proxy_url,
                proxy_port,
                target_url: request.target_url,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

/// Stops the active recording session.
///
/// # Arguments
///
/// * `recording_manager` - The recording session manager.
#[axum::debug_handler]
pub async fn stop_recording(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
) -> Result<Json<ApiResponse<StopRecordingResponse>>, StatusCode> {
    match recording_manager.stop_recording().await {
        Ok((session_id, endpoints)) => {
            let response = StopRecordingResponse {
                session_id,
                captured_count: endpoints.len(),
                endpoints,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

/// Gets the status of the current recording session.
///
/// # Arguments
///
/// * `recording_manager` - The recording session manager.
#[axum::debug_handler]
pub async fn get_recording_status(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
) -> Result<Json<ApiResponse<RecordingStatusResponse>>, StatusCode> {
    let (is_active, session_id, proxy_url, proxy_port, target_url, captured_count) =
        recording_manager.get_status().await;

    let response = RecordingStatusResponse {
        is_active,
        session_id,
        proxy_url,
        proxy_port,
        target_url,
        captured_count,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Generates a service definition from the captured requests.
///
/// # Arguments
///
/// * `request` - The request containing service name and description.
/// * `recording_manager` - The recording session manager.
/// * `simulator` - The API simulator manager.
#[axum::debug_handler]
pub async fn generate_service_from_recording(
    Extension(recording_manager): Extension<Arc<RecordingSessionManager>>,
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(request): Json<GenerateServiceRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Validate service name
    if let Err(e) = validation::validate_service_name(&request.service_name) {
        return Ok(Json(ApiResponse::error(e.message)));
    }

    // Stop the recording and get the endpoints
    let (session_id, endpoints) = match recording_manager.stop_recording().await {
        Ok(result) => result,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    if endpoints.is_empty() {
        return Ok(Json(ApiResponse::error(
            "No requests were captured during recording".to_string(),
        )));
    }

    let endpoint_count = endpoints.len();

    // Create a service definition from the captured endpoints
    let service_def = ServiceDefinition {
        name: request.service_name.clone(),
        version: Some("1.0.0".to_string()),
        description: request.description.or(Some(format!(
            "Service generated from recording session {}",
            session_id
        ))),
        server: Some(ServerConfig {
            port: None,
            base_path: "/".to_string(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        }),
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Some(endpoints),
        graphql: None,
        behavior: None,
        twin: None,
    };

    // Convert to YAML
    let yaml = match serde_yaml::to_string(&service_def) {
        Ok(y) => y,
        Err(e) => {
            return Ok(Json(ApiResponse::error(format!(
                "Failed to serialize service: {}",
                e
            ))))
        }
    };

    // Resolve safe path using simulator
    let file_path = match simulator.resolve_service_path(&format!("{}.yaml", request.service_name))
    {
        Ok(path) => path,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Check if file already exists
    if simulator.service_file_exists(&file_path) {
        return Ok(Json(ApiResponse::error(format!(
            "Service file '{}' already exists",
            request.service_name
        ))));
    }

    // Write the service definition to file
    match simulator.save_service_file(&file_path, &yaml) {
        Ok(_) => {
            // Apply the service to the running simulator
            if let Err(e) = simulator.apply_service_definition(service_def).await {
                // Clean up the file if applying fails
                let _ = simulator.delete_service_file(&file_path);
                return Ok(Json(ApiResponse::error(format!(
                    "Failed to apply service: {}",
                    e
                ))));
            }

            Ok(Json(ApiResponse::success(format!(
                "Service '{}' generated successfully from {} captured endpoints",
                request.service_name, endpoint_count
            ))))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!(
            "Failed to write service file: {}",
            e
        )))),
    }
}
