use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use crate::errors::{PulseError, PulseResult};
use super::AiProvider;

/// OpenAI based provider used when remote generation is desired.
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { client: Client::new(), api_key, model }
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
    async fn generate_yaml(&self, prompt: &str) -> PulseResult<String> {
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
            .map_err(|e| PulseError::runtime_error(e.to_string(), None::<String>))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(PulseError::runtime_error(
                format!("OpenAI request failed: {}", status),
                None::<String>,
            ));
        }

        let resp_json: ChatCompletionResponse = resp
            .json()
            .await
            .map_err(|e| PulseError::runtime_error(e.to_string(), None::<String>))?;

        let content = resp_json
            .choices
            .into_iter()
            .flat_map(|c| c.message.content)
            .next()
            .unwrap_or_default();

        Ok(content)
    }
}
