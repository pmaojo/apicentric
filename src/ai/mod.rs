use async_trait::async_trait;
use crate::errors::ApicentricResult;

pub mod local;
pub mod openai;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_yaml(&self, prompt: &str) -> ApicentricResult<String>;
}

pub use local::LocalAiProvider;
pub use openai::OpenAiProvider;
