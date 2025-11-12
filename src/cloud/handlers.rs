//! Axum handlers for the cloud API.
//!
//! This module provides handlers for listing, loading, and saving services.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::ApicentricError;
use crate::simulator::{ApiSimulatorManager, ServiceInfo};

// --- Generic API Response ---

/// A generic API response.
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self { success: true, data: Some(data), error: None })
    }
    pub fn error(message: String) -> (StatusCode, Json<Self>) {
        (StatusCode::BAD_REQUEST, Json(Self { success: false, data: None, error: Some(message) }))
    }
    pub fn server_error(message: String) -> (StatusCode, Json<Self>) {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Self { success: false, data: None, error: Some(message) }))
    }
}

// --- Request/Response DTOs ---

#[derive(Deserialize)]
pub struct StartRequest {
    #[serde(rename = "servicesDir", default)]
    pub services_dir: Option<String>,
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub p2p: bool,
}

#[derive(Deserialize)]
pub struct StopRequest {
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize)]
pub struct StatusQuery {
    #[serde(default)]
    pub detailed: bool,
}

#[derive(Deserialize)]
pub struct ValidateRequest {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Deserialize)]
pub struct SetScenarioRequest {
    pub scenario: String,
}

#[derive(Deserialize)]
pub struct ImportRequest {
    pub input: String,
    pub output: String,
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub input: String,
    pub output: String,
    pub format: String,
}

#[derive(Deserialize)]
pub struct NewServiceRequest {
    pub output: String,
}

#[derive(Deserialize)]
pub struct NewGraphqlRequest {
    pub name: String,
    pub output: String,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub service: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub method: Option<String>,
    pub route: Option<String>,
    pub status: Option<u16>,
    pub output: Option<String>,
}
fn default_limit() -> usize { 20 }

#[derive(Deserialize)]
pub struct DockerizeRequest {
    pub services: Vec<String>,
    pub output: String,
}

#[derive(Deserialize)]
pub struct AiGenerateRequest {
    pub prompt: String,
}

// --- Handlers ---

pub async fn start_simulator(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(payload): Json<StartRequest>,
) -> Result<Json<ApiResponse<ServiceInfo>>, (StatusCode, Json<ApiResponse<()>>)> {
    if let Some(dir) = payload.services_dir {
        simulator.config.services_dir = dir.into();
    }
    if payload.p2p {
        simulator.enable_p2p(true).await;
    }
    simulator.start().await.map_err(|e| ApiResponse::server_error(e.to_string()))?;
    Ok(ApiResponse::success(ServiceInfo { name: "Simulator".into(), port: 0, base_path: "".into(), endpoints_count: 0, is_running: true }))
}

pub async fn stop_simulator(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(_payload): Json<StopRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    simulator.stop().await.map_err(|e| ApiResponse::server_error(e.to_string()))?;
    Ok(ApiResponse::success(()))
}

pub async fn get_status(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Query(_query): Query<StatusQuery>,
) -> Result<Json<ApiResponse<crate::simulator::SimulatorStatus>>, (StatusCode, Json<ApiResponse<()>>)> {
    let status = simulator.get_status().await;
    Ok(ApiResponse::success(status))
}

pub async fn validate_services(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(_payload): Json<ValidateRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let valid_services = simulator.validate_configurations().map_err(|e| ApiResponse::server_error(e.to_string()))?;
    Ok(ApiResponse::success(valid_services))
}

pub async fn set_scenario(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(payload): Json<SetScenarioRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    simulator.set_scenario(Some(payload.scenario)).await.map_err(|e| ApiResponse::server_error(e.to_string()))?;
    Ok(ApiResponse::success(()))
}

pub async fn import_service(
    Json(_payload): Json<ImportRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Placeholder logic
    Err(ApiResponse::server_error("Not yet implemented".into()))
}

pub async fn export_service(
    Json(_payload): Json<ExportRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Placeholder logic
    Err(ApiResponse::server_error("Not yet implemented".into()))
}

pub async fn new_service(
    Json(_payload): Json<NewServiceRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Placeholder logic
    Err(ApiResponse::server_error("Not yet implemented".into()))
}

pub async fn new_graphql_service(
    Json(_payload): Json<NewGraphqlRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Placeholder logic
    Err(ApiResponse::server_error("Not yet implemented".into()))
}

pub async fn get_logs(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<ApiResponse<Vec<crate::simulator::log::RequestLogEntry>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let logs = simulator.query_logs(Some(&query.service), query.route.as_deref(), query.method.as_deref(), query.status, query.limit).await;
    Ok(ApiResponse::success(logs))
}

pub async fn dockerize_service(
    Json(_payload): Json<DockerizeRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Placeholder logic
    Err(ApiResponse::server_error("Not yet implemented".into()))
}

pub async fn ai_generate(
    State(simulator): State<Arc<ApiSimulatorManager>>,
    Json(payload): Json<AiGenerateRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let cfg = simulator.config.clone();
     let ai_cfg = match cfg.ai {
        Some(cfg) => cfg,
        None => {
            return Err(ApiResponse::server_error("AI provider not configured".to_string()));
        }
    };
    
    let provider: Box<dyn crate::ai::AiProvider> = match ai_cfg.provider {
        crate::config::AiProviderKind::Openai => {
             let key = ai_cfg.api_key.ok_or_else(|| ApicentricError::config_error("Missing OpenAI key", None::<String>)).map_err(|e| ApiResponse::server_error(e.to_string()))?;
             let model = ai_cfg.model.unwrap_or_else(|| "gpt-3.5-turbo".into());
             Box::new(crate::ai::OpenAiProvider::new(key, model))
        },
        crate::config::AiProviderKind::Gemini => {
            let key = ai_cfg.api_key.ok_or_else(|| ApicentricError::config_error("Missing Gemini key", None::<String>)).map_err(|e| ApiResponse::server_error(e.to_string()))?;
            let model = ai_cfg.model.unwrap_or_else(|| "gemini-pro".into());
            Box::new(crate::ai::GeminiAiProvider::new(key, model))
        },
        _ => return Err(ApiResponse::server_error("Unsupported AI provider".into())),
    };
    
    let yaml = provider.generate_yaml(&payload.prompt).await.map_err(|e| ApiResponse::server_error(e.to_string()))?;
    Ok(ApiResponse::success(yaml))
}
