//! Provides a common interface for AI providers.
//!
//! This module defines the `AiProvider` trait, which is implemented by AI
//! providers that can generate YAML from a prompt.

use async_trait::async_trait;
use crate::errors::ApicentricResult;

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
