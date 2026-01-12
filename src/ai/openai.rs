//! An AI provider that uses the OpenAI API to generate YAML from a prompt.
//!
//! This module provides an `OpenAiProvider` that can be used to generate YAML
//! from a prompt using the OpenAI API.

use super::AiProvider;
use crate::errors::{ApicentricError, ApicentricResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

/// An AI provider that uses the OpenAI API.
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    /// Creates a new `OpenAiProvider`.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The OpenAI API key.
    /// * `model` - The name of the model to use.
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Deserialize)]
struct ChoiceMessage {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChoiceMessage>,
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    /// Generates YAML from a prompt using the OpenAI API.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to use for generating the YAML.
    ///
    /// # Returns
    ///
    /// The generated YAML.
    async fn generate_yaml(&self, prompt: &str) -> ApicentricResult<String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are an assistant that outputs YAML service definitions"},
                {"role": "user", "content": prompt}
            ]
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ApicentricError::runtime_error(
                format!("OpenAI request failed: {}", status),
                None::<String>,
            ));
        }

        let resp_json: ChatCompletionResponse = resp
            .json()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

        let content = resp_json
            .choices
            .into_iter()
            .flat_map(|c| c.message.content)
            .next()
            .unwrap_or_default();

        Ok(content)
    }
}
