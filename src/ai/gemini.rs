//! An AI provider that uses the Google Gemini API to generate YAML from a prompt.
//!
//! This module provides a `GeminiAiProvider` that can be used to generate YAML
//! from a prompt using the Google Gemini API.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::errors::{ApicentricError, ApicentricResult};
use super::AiProvider;

/// An AI provider that uses the Google Gemini API.
pub struct GeminiAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiAiProvider {
    /// Creates a new `GeminiAiProvider`.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Google Gemini API key.
    /// * `model` - The name of the model to use.
    pub fn new(api_key: String, model: String) -> Self {
        Self { client: Client::new(), api_key, model }
    }
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: String,
}

#[async_trait]
impl AiProvider for GeminiAiProvider {
    /// Generates YAML from a prompt using the Google Gemini API.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to use for generating the YAML.
    ///
    /// # Returns
    ///
    /// The generated YAML.
    async fn generate_yaml(&self, prompt: &str) -> ApicentricResult<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        // First request: Generate initial YAML
        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!("Generate a YAML service definition for API simulation based on this prompt: {}. You can include markdown formatting and explanations.", prompt)
                }]
            }]
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ApicentricError::runtime_error(
                format!("Gemini request failed: {}", status),
                None::<String>,
            ));
        }

        let resp_json: GeminiResponse = resp
            .json()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

        let raw_content = resp_json
            .candidates
            .into_iter()
            .flat_map(|c| c.content.parts)
            .map(|p| p.text)
            .collect::<Vec<_>>()
            .join("");

        // Second request: Clean and correct the YAML
        let correction_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!("Take this YAML content and clean it up to be a valid ServiceDefinition object. Remove any markdown formatting, code blocks, or explanatory text. Ensure it has these required fields: name (string), server (object with port and base_path starting with '/'), endpoints (array of endpoint objects with method, path, and responses as HashMap<u16, ResponseDefinition>). Each response must have content_type and body fields. Output only the clean YAML content as a single object, not an array.\n\nRaw content:\n{}", raw_content)
                }]
            }]
        });

        let correction_resp = self
            .client
            .post(&url)
            .json(&correction_body)
            .send()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

        let correction_status = correction_resp.status();
        if !correction_status.is_success() {
            return Err(ApicentricError::runtime_error(
                format!("Gemini correction request failed: {}", correction_status),
                None::<String>,
            ));
        }

        let correction_resp_json: GeminiResponse = correction_resp
            .json()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

        let clean_content = correction_resp_json
            .candidates
            .into_iter()
            .flat_map(|c| c.content.parts)
            .map(|p| p.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(clean_content)
    }
}