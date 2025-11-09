use crate::errors::ValidationError;
use crate::validation::{ConfigValidator, ValidationUtils};
use serde::{Deserialize, Serialize};
use url::Url;

/// Server configuration for a service
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub base_path: String,
    /// Optional base URL to proxy requests when no local endpoint matches
    #[serde(default)]
    pub proxy_base_url: Option<String>,
    #[serde(default)]
    pub cors: Option<CorsConfig>,
    #[serde(default)]
    pub record_unknown: bool,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    #[serde(default)]
    pub methods: Option<Vec<String>>,
    #[serde(default)]
    pub headers: Option<Vec<String>>,
}

impl ConfigValidator for ServerConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate base path
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.base_path, "server.base_path")
        {
            errors.push(e);
        }

        if !self.base_path.starts_with('/') {
            errors.push(ValidationError {
                field: "server.base_path".to_string(),
                message: "Base path must start with '/'".to_string(),
                suggestion: Some("Ensure base path starts with '/', e.g., '/api/v1'".to_string()),
            });
        }

        // Validate port if specified
        if let Some(port) = self.port {
            if port < 1024 {
                errors.push(ValidationError {
                    field: "server.port".to_string(),
                    message: "Port should be >= 1024 to avoid system ports".to_string(),
                    suggestion: Some("Use ports 1024 or higher".to_string()),
                });
            }
        }

        // Validate proxy_base_url if specified
        if let Some(url) = &self.proxy_base_url {
            if Url::parse(url).is_err() {
                errors.push(ValidationError {
                    field: "server.proxy_base_url".to_string(),
                    message: "Invalid URL for proxy_base_url".to_string(),
                    suggestion: Some(
                        "Provide a valid URL such as 'http://example.com'".to_string(),
                    ),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
