use super::{AiConfig, AiProviderKind, ApicentricConfig};
use crate::errors::ValidationError;
use crate::validation::ConfigValidator;

impl ConfigValidator for AiConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        match self.provider {
            AiProviderKind::Local => {
                if self.model_path.as_deref().unwrap_or("").is_empty() {
                    errors.push(ValidationError::new(
                        "ai.model_path",
                        "model_path is required for local provider",
                    ));
                }
            }
            AiProviderKind::Openai => {
                if self.api_key.as_deref().unwrap_or("").is_empty() {
                    errors.push(ValidationError::new(
                        "ai.api_key",
                        "api_key is required for openai provider",
                    ));
                }
            }
            AiProviderKind::Gemini => {
                // Gemini can use GEMINI_API_KEY from environment or ai.api_key from config
                if self.api_key.as_deref().unwrap_or("").is_empty() && std::env::var("GEMINI_API_KEY").is_err() {
                    errors.push(ValidationError::new(
                        "ai.api_key",
                        "api_key is required for gemini provider (or set GEMINI_API_KEY environment variable)",
                    ));
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl ConfigValidator for ApicentricConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Validate AI config if present
        if let Some(ref ai) = self.ai {
            if let Err(mut ai_errors) = ai.validate() {
                errors.append(&mut ai_errors);
            }
        }
        
        // Validate simulator config if present
        if let Some(ref simulator) = self.simulator {
            if let Err(mut simulator_errors) = simulator.validate() {
                errors.append(&mut simulator_errors);
            }
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_config_validation_gemini() {
        let config = AiConfig {
            provider: AiProviderKind::Gemini,
            model_path: None,
            api_key: Some("test-key".to_string()),
            model: Some("gemini-2.5-flash".to_string()),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn ai_config_validation_openai_missing_key() {
        let config = AiConfig {
            provider: AiProviderKind::Openai,
            model_path: None,
            api_key: None,
            model: Some("gpt-4".to_string()),
        };
        assert!(config.validate().is_err());
    }
}

