//! Provides a common interface for AI providers.
//!
//! This module defines the `AiProvider` trait, which is implemented by AI
//! providers that can generate YAML from a prompt.

use crate::config::AiProviderKind;
use crate::{ApicentricError, ApicentricResult, Context};
use async_trait::async_trait;

pub mod gemini;
pub mod local;
pub mod openai;

/// A trait for AI providers that can generate YAML from a prompt.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Generates YAML from a prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to use for generating the YAML.
    ///
    /// # Returns
    ///
    /// The generated YAML.
    async fn generate_yaml(&self, prompt: &str) -> ApicentricResult<String>;
}

pub use gemini::GeminiAiProvider;
pub use local::LocalAiProvider;
pub use openai::OpenAiProvider;

pub async fn generate_service(context: &Context, prompt: &str) -> ApicentricResult<String> {
    let cfg = context.config();
    let ai_cfg = match &cfg.ai {
        Some(cfg) => cfg,
        None => {
            return Err(ApicentricError::config_error(
                "AI provider not configured",
                Some("Add an 'ai' section to apicentric.json"),
            ))
        }
    };

    // Build provider based on configuration
    let provider: Box<dyn AiProvider> = match ai_cfg.provider {
        AiProviderKind::Local => {
            let path = ai_cfg
                .model_path
                .clone()
                .unwrap_or_else(|| "model.bin".to_string());
            Box::new(LocalAiProvider::new(path))
        }
        AiProviderKind::Openai => {
            let key = ai_cfg.api_key.clone().ok_or_else(|| {
                ApicentricError::config_error(
                    "OpenAI API key missing",
                    Some("Set ai.api_key in apicentric.json"),
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
                    ApicentricError::config_error(
                        "Gemini API key missing",
                        Some(
                            "Set GEMINI_API_KEY environment variable or ai.api_key in apicentric.json",
                        ),
                    )
                })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
            Box::new(GeminiAiProvider::new(key, model))
        }
    };

    provider.generate_yaml(prompt).await
}
