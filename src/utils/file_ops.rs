use async_trait::async_trait;
use std::path::Path;

/// Abstraction over file reading to enable mock implementations.
#[async_trait]
pub trait FileReader: Send + Sync {
    async fn read_to_string(&self, path: &Path) -> std::io::Result<String>;
}

/// Default production implementation using Tokio's asynchronous file APIs.
pub struct TokioFileReader;

#[async_trait]
impl FileReader for TokioFileReader {
    async fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        tokio::fs::read_to_string(path).await
    }
}
