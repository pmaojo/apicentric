//! AI Service Generator
//!
//! Handles the generation of service definitions using AI providers.

#![cfg(feature = "gui")]
#![allow(dead_code)]

use apicentric::ai::{AiProvider, GeminiAiProvider, LocalAiProvider, OpenAiProvider};
use apicentric::config::AiProviderKind;
use apicentric::{ApicentricError, ApicentricResult};
use std::path::Path;

/// AI Service Generator that coordinates AI-powered service definition generation
pub struct AiServiceGenerator {
    provider: Box<dyn AiProvider>,
}

impl AiServiceGenerator {
    /// Create a new AI service generator from configuration
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the apicentric.json configuration file
    ///
    /// # Returns
    ///
    /// A new AiServiceGenerator instance or an error if configuration is invalid
    pub fn from_config(config_path: &Path) -> ApicentricResult<Self> {
        let cfg = apicentric::config::load_config(config_path)?;
        let ai_cfg = cfg.ai.ok_or_else(|| {
            ApicentricError::config_error(
                "AI provider not configured",
                Some("Add an 'ai' section to apicentric.json"),
            )
        })?;

        let provider = Self::create_provider(&ai_cfg)?;
        Ok(Self { provider })
    }

    /// Create a new AI service generator with a specific provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The AI provider to use
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self { provider }
    }

    /// Create an AI provider from configuration
    fn create_provider(
        ai_cfg: &apicentric::config::AiConfig,
    ) -> ApicentricResult<Box<dyn AiProvider>> {
        match ai_cfg.provider {
            AiProviderKind::Local => {
                let path = ai_cfg
                    .model_path
                    .clone()
                    .unwrap_or_else(|| "model.bin".to_string());
                Ok(Box::new(LocalAiProvider::new(path)))
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
                Ok(Box::new(OpenAiProvider::new(key, model)))
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
                Ok(Box::new(GeminiAiProvider::new(key, model)))
            }
        }
    }

    /// Generate a service definition from a prompt
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user's prompt describing the desired service
    ///
    /// # Returns
    ///
    /// The generated YAML service definition or an error
    pub async fn generate_from_prompt(&self, prompt: &str) -> ApicentricResult<String> {
        // Validate prompt
        if prompt.trim().is_empty() {
            return Err(ApicentricError::runtime_error(
                "Prompt cannot be empty",
                Some("Please provide a description of the service you want to generate"),
            ));
        }

        // Generate YAML using the provider
        let yaml = self.provider.generate_yaml(prompt).await?;

        // Return the generated YAML
        // Note: Validation will be done separately before applying
        Ok(yaml)
    }

    /// Validate YAML content before applying
    ///
    /// # Arguments
    ///
    /// * `yaml` - The YAML content to validate
    ///
    /// # Returns
    ///
    /// A validation result with detailed error information if invalid
    pub fn validate_yaml(&self, yaml: &str) -> ApicentricResult<ValidationResult> {
        // Parse YAML to check syntax
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(yaml);
        
        match parsed {
            Ok(value) => {
                // Check for required fields
                let mut errors = Vec::new();
                
                if value.get("name").is_none() {
                    errors.push(ValidationError {
                        line: None,
                        message: "Missing required field: 'name'".to_string(),
                    });
                }
                
                if value.get("server").is_none() {
                    errors.push(ValidationError {
                        line: None,
                        message: "Missing required field: 'server'".to_string(),
                    });
                }
                
                if value.get("endpoints").is_none() {
                    errors.push(ValidationError {
                        line: None,
                        message: "Missing required field: 'endpoints'".to_string(),
                    });
                }
                
                if errors.is_empty() {
                    Ok(ValidationResult {
                        valid: true,
                        errors: Vec::new(),
                    })
                } else {
                    Ok(ValidationResult {
                        valid: false,
                        errors,
                    })
                }
            }
            Err(e) => {
                // Parse error - extract line number if available
                let error_msg = e.to_string();
                let line = extract_line_number(&error_msg);
                
                Ok(ValidationResult {
                    valid: false,
                    errors: vec![ValidationError {
                        line,
                        message: format!("YAML syntax error: {}", error_msg),
                    }],
                })
            }
        }
    }

    /// Get a user-friendly error message for configuration issues
    ///
    /// # Arguments
    ///
    /// * `error` - The error to format
    ///
    /// # Returns
    ///
    /// A user-friendly error message with suggestions
    pub fn format_error_message(error: &ApicentricError) -> String {
        let error_str = error.to_string();
        
        if error_str.contains("AI provider not configured") {
            format!(
                "❌ AI Provider Not Configured\n\n\
                 The AI generation feature requires configuration.\n\n\
                 To fix this:\n\
                 1. Open apicentric.json\n\
                 2. Add an 'ai' section with your provider settings\n\n\
                 Example:\n\
                 {{\n\
                   \"ai\": {{\n\
                     \"provider\": \"openai\",\n\
                     \"api_key\": \"your-api-key\",\n\
                     \"model\": \"gpt-3.5-turbo\"\n\
                   }}\n\
                 }}"
            )
        } else if error_str.contains("API key missing") {
            format!(
                "❌ API Key Missing\n\n\
                 Your AI provider requires an API key.\n\n\
                 To fix this:\n\
                 1. Open apicentric.json\n\
                 2. Add your API key to the 'ai.api_key' field\n\n\
                 For OpenAI: Get your key from https://platform.openai.com/api-keys\n\
                 For Gemini: Set GEMINI_API_KEY environment variable or add to config"
            )
        } else if error_str.contains("Network") || error_str.contains("connection") {
            format!(
                "❌ Network Error\n\n\
                 Failed to connect to the AI provider.\n\n\
                 Possible causes:\n\
                 • No internet connection\n\
                 • AI provider service is down\n\
                 • Firewall blocking the connection\n\n\
                 Please check your connection and try again."
            )
        } else if error_str.contains("Rate limit") {
            format!(
                "⚠️ Rate Limit Exceeded\n\n\
                 You've made too many requests to the AI provider.\n\n\
                 Please wait a moment and try again."
            )
        } else {
            format!("❌ AI Generation Error\n\n{}", error_str)
        }
    }
}

/// Result of YAML validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

/// A validation error with optional line number
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub message: String,
}

/// Extract line number from error message if present
fn extract_line_number(error_msg: &str) -> Option<usize> {
    // Try to extract line number from error message
    // Common formats: "line 5", "at line 5", "5:10"
    if let Some(pos) = error_msg.find("line") {
        let after_line = &error_msg[pos + 4..];
        if let Some(num_str) = after_line.split_whitespace().next() {
            if let Ok(line) = num_str.trim_matches(|c: char| !c.is_numeric()).parse() {
                return Some(line);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use apicentric::config::AiConfig;

    struct MockProvider {
        response: String,
    }

    #[async_trait::async_trait]
    impl AiProvider for MockProvider {
        async fn generate_yaml(&self, _prompt: &str) -> ApicentricResult<String> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_generator_with_mock_provider() {
        let mock = Box::new(MockProvider {
            response: "name: test\nversion: \"1.0\"".to_string(),
        });
        let generator = AiServiceGenerator::new(mock);

        let result = generator.generate_from_prompt("test prompt").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("name: test"));
    }

    #[tokio::test]
    async fn test_generator_empty_prompt() {
        let mock = Box::new(MockProvider {
            response: "name: test".to_string(),
        });
        let generator = AiServiceGenerator::new(mock);

        let result = generator.generate_from_prompt("").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_generator_whitespace_prompt() {
        let mock = Box::new(MockProvider {
            response: "name: test".to_string(),
        });
        let generator = AiServiceGenerator::new(mock);

        let result = generator.generate_from_prompt("   \n  \t  ").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_openai() {
        let ai_cfg = AiConfig {
            provider: AiProviderKind::Openai,
            api_key: Some("test-key".to_string()),
            model: Some("gpt-4".to_string()),
            model_path: None,
        };

        let result = AiServiceGenerator::create_provider(&ai_cfg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_provider_openai_missing_key() {
        let ai_cfg = AiConfig {
            provider: AiProviderKind::Openai,
            api_key: None,
            model: Some("gpt-4".to_string()),
            model_path: None,
        };

        let result = AiServiceGenerator::create_provider(&ai_cfg);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("API key missing"));
    }

    #[test]
    fn test_create_provider_local() {
        let ai_cfg = AiConfig {
            provider: AiProviderKind::Local,
            api_key: None,
            model: None,
            model_path: Some("model.bin".to_string()),
        };

        let result = AiServiceGenerator::create_provider(&ai_cfg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_provider_local_default_path() {
        let ai_cfg = AiConfig {
            provider: AiProviderKind::Local,
            api_key: None,
            model: None,
            model_path: None,
        };

        let result = AiServiceGenerator::create_provider(&ai_cfg);
        assert!(result.is_ok());
    }
}
