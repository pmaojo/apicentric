use async_trait::async_trait;
use crate::errors::PulseResult;

pub mod local;
pub mod openai;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_yaml(&self, prompt: &str) -> PulseResult<String>;
}

pub use local::LocalAiProvider;
pub use openai::OpenAiProvider;
