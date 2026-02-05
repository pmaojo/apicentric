//! AI handlers for the cloud API.
//!
//! This module provides handlers for AI-assisted service generation and validation.

use axum::{
    extract::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::cloud::error::{ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::response::ApiResponse;
use crate::simulator::ServiceDefinition;

/// Request to generate a service using AI.
#[derive(Deserialize)]
pub struct AiGenerateRequest {
    /// The natural language prompt describing the service.
    pub prompt: String,
    /// Optional AI provider to use (openai, gemini, local).
    pub provider: Option<String>,
}

/// Response from AI generation.
#[derive(Serialize)]
pub struct AiGenerateResponse {
    /// The generated YAML service definition.
    pub yaml: String,
    /// Any validation errors found in the generated YAML.
    pub validation_errors: Vec<String>,
}

/// Request to validate YAML.
#[derive(Deserialize)]
pub struct AiValidateRequest {
    /// The YAML content to validate.
    pub yaml: String,
}

/// Response from YAML validation.
#[derive(Serialize)]
pub struct AiValidateResponse {
    /// Whether the YAML is valid.
    pub is_valid: bool,
    /// Any validation errors found.
    pub errors: Vec<String>,
}

/// Response for AI configuration status.
#[derive(Serialize)]
pub struct AiConfigResponse {
    /// Whether AI is configured.
    pub is_configured: bool,
    /// The configured provider (if any).
    pub provider: Option<String>,
    /// The configured model (if any).
    pub model: Option<String>,
    /// Any configuration issues.
    pub issues: Vec<String>,
}

/// Generates a service definition using AI from a natural language prompt.
///
/// # Arguments
///
/// * `request` - The AI generation request containing the prompt.
#[axum::debug_handler]
pub async fn ai_generate(
    Json(request): Json<AiGenerateRequest>,
) -> Result<Json<ApiResponse<AiGenerateResponse>>, ApiError> {
    use crate::ai::{AiProvider, GeminiAiProvider, LocalAiProvider, OpenAiProvider};
    use crate::config::{load_config, AiProviderKind};
    use crate::validation::ConfigValidator;
    use std::path::Path;

    // Validate prompt is not empty
    if request.prompt.trim().is_empty() {
        return Err(ApiError::bad_request(
            ApiErrorCode::InvalidParameter,
            "Prompt cannot be empty",
        ));
    }

    // Load configuration
    let cfg = match load_config(Path::new("apicentric.json")) {
        Ok(cfg) => cfg,
        Err(_) => {
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::ConfigLoadError,
                "Failed to load configuration file apicentric.json",
            ));
        }
    };
    let ai_cfg = match &cfg.ai {
        Some(cfg) => cfg,
        None => {
            return Err(ErrorResponse::ai_not_configured().into());
        }
    };

    // Determine which provider to use (from request or config)
    let provider_kind = if let Some(ref provider_str) = request.provider {
        match provider_str.to_lowercase().as_str() {
            "openai" => AiProviderKind::Openai,
            "gemini" => AiProviderKind::Gemini,
            "local" => AiProviderKind::Local,
            _ => {
                return Err(ApiError::bad_request(
                    ApiErrorCode::InvalidAiProvider,
                    format!(
                        "Unknown AI provider: {}. Use 'openai', 'gemini', or 'local'",
                        provider_str
                    ),
                ));
            }
        }
    } else {
        ai_cfg.provider.clone()
    };

    // Build provider based on configuration
    let provider: Box<dyn AiProvider> = match provider_kind {
        AiProviderKind::Local => {
            let path = ai_cfg
                .model_path
                .clone()
                .unwrap_or_else(|| "model.bin".to_string());
            Box::new(LocalAiProvider::new(path))
        }
        AiProviderKind::Openai => {
            let key = ai_cfg.api_key.clone().ok_or_else(|| {
                ApiError::bad_request(
                    ApiErrorCode::AiNotConfigured,
                    "OpenAI API key missing. Set ai.api_key in apicentric.json",
                )
            })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
            Box::new(OpenAiProvider::new(key, model))
        }
        AiProviderKind::Gemini => {
            let key = std::env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| ai_cfg.api_key.clone())
                .ok_or_else(|| {
                    ApiError::bad_request(
                        ApiErrorCode::AiNotConfigured,
                        "Gemini API key missing. Set GEMINI_API_KEY environment variable or ai.api_key in apicentric.json",
                    )
                })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
            Box::new(GeminiAiProvider::new(key, model))
        }
    };

    // Generate YAML from prompt
    let yaml = match provider.generate_yaml(&request.prompt).await {
        Ok(yaml) => yaml,
        Err(e) => {
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::AiGenerationFailed,
                format!("AI generation failed: {}", e),
            ));
        }
    };

    // Validate the generated YAML
    let validation_errors = match serde_yaml::from_str::<ServiceDefinition>(&yaml) {
        Ok(def) => {
            // Validate the service definition
            match def.validate() {
                Ok(_) => Vec::new(),
                Err(errors) => errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect(),
            }
        }
        Err(e) => vec![format!("YAML parsing error: {}", e)],
    };

    let response = AiGenerateResponse {
        yaml,
        validation_errors,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Validates a YAML service definition.
///
/// # Arguments
///
/// * `request` - The validation request containing the YAML.
#[axum::debug_handler]
pub async fn ai_validate(
    Json(request): Json<AiValidateRequest>,
) -> Result<Json<ApiResponse<AiValidateResponse>>, StatusCode> {
    use crate::validation::ConfigValidator;

    let errors = match serde_yaml::from_str::<ServiceDefinition>(&request.yaml) {
        Ok(def) => {
            // Validate the service definition
            match def.validate() {
                Ok(_) => Vec::new(),
                Err(validation_errors) => validation_errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect(),
            }
        }
        Err(e) => vec![format!("YAML parsing error: {}", e)],
    };

    let response = AiValidateResponse {
        is_valid: errors.is_empty(),
        errors,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Checks the AI configuration status.
#[axum::debug_handler]
pub async fn ai_config_status() -> Result<Json<ApiResponse<AiConfigResponse>>, StatusCode> {
    use crate::config::{load_config, AiProviderKind};
    use std::path::Path;

    // Load configuration
    let cfg = match load_config(Path::new("apicentric.json")) {
        Ok(cfg) => cfg,
        Err(_) => {
            return Ok(Json(ApiResponse::error(
                "Failed to load configuration file apicentric.json".to_string(),
            )));
        }
    };
    let mut issues = Vec::new();

    let (is_configured, provider, model) = match &cfg.ai {
        Some(ai_cfg) => {
            let provider_str = match ai_cfg.provider {
                AiProviderKind::Openai => "openai",
                AiProviderKind::Gemini => "gemini",
                AiProviderKind::Local => "local",
            };

            // Check for provider-specific configuration issues
            match ai_cfg.provider {
                AiProviderKind::Openai => {
                    if ai_cfg.api_key.is_none() {
                        issues.push("OpenAI API key not configured".to_string());
                    }
                }
                AiProviderKind::Gemini => {
                    if ai_cfg.api_key.is_none() && std::env::var("GEMINI_API_KEY").is_err() {
                        issues.push(
                            "Gemini API key not configured (set GEMINI_API_KEY or ai.api_key)"
                                .to_string(),
                        );
                    }
                }
                AiProviderKind::Local => {
                    if let Some(ref path) = ai_cfg.model_path {
                        if !std::path::Path::new(path).exists() {
                            issues.push(format!("Local model file not found: {}", path));
                        }
                    } else {
                        issues.push("Local model path not configured".to_string());
                    }
                }
            }

            (
                issues.is_empty(),
                Some(provider_str.to_string()),
                ai_cfg.model.clone(),
            )
        }
        None => {
            issues.push("AI configuration not found in apicentric.json".to_string());
            (false, None, None)
        }
    };

    let response = AiConfigResponse {
        is_configured,
        provider,
        model,
        issues,
    };

    Ok(Json(ApiResponse::success(response)))
}
