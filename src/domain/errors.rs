<<<<<<< HEAD
lmuse thiserror::Error;
=======
use thiserror::Error;
>>>>>>> origin/main

#[derive(Debug, Error)]
pub enum ApicentricError {
    #[error("Test error: {0}")]
    Test(String),
    #[error("Filesystem error: {0}")]
    Fs(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Config error: {0}")]
    Config(String),
}

pub type ApicentricResult<T> = Result<T, ApicentricError>;
