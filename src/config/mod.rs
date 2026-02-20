use serde::{Deserialize, Serialize};

pub mod repository;
pub mod validation;

pub use repository::{
    init_config, load_config, save_config, ConfigRepository, FileConfigRepository,
};

/// Main configuration structure for Apicentric
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ApicentricConfig {
    /// AI generation configuration
    #[serde(default)]
    pub ai: Option<AiConfig>,

    /// API Simulator configuration (the main feature)
    #[serde(default)]
    pub simulator: Option<crate::simulator::config::SimulatorConfig>,
}

impl ApicentricConfig {
    /// Redacts sensitive fields (like API keys) with a mask.
    pub fn redact_sensitive_fields(&mut self) {
        if let Some(ai) = &mut self.ai {
            if ai.api_key.is_some() {
                ai.api_key = Some("********".to_string());
            }
        }
    }

    /// Merges the configuration with the current configuration, restoring redacted fields.
    pub fn merge_with_current(&mut self, current: &ApicentricConfig) {
        if let Some(ai) = &mut self.ai {
            if let Some(current_ai) = &current.ai {
                // If the new key is masked, restore the old key
                if ai.api_key.as_deref() == Some("********") {
                    ai.api_key = current_ai.api_key.clone();
                }
            }
        }
    }
}

// ============================================================================
// AI Configuration
// ============================================================================

/// Configuration for AI assisted code generation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiConfig {
    /// Provider to use for text generation.
    pub provider: AiProviderKind,
    /// Path to the local model when using the `local` provider.
    #[serde(default)]
    pub model_path: Option<String>,
    /// API key when using the `openai` provider.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Optional model identifier for remote providers.
    #[serde(default)]
    pub model: Option<String>,
}

/// Supported AI providers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderKind {
    /// Use a local language model such as `llama-rs`.
    Local,
    /// Use the OpenAI API.
    Openai,
    /// Use the Google Gemini API.
    Gemini,
}

// ============================================================================
// Legacy Types (for backward compatibility only)
// ============================================================================

// ============================================================================
// Legacy Types (minimal for CLI compatibility only)
// ============================================================================

/// Execution modes (for CLI only)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    CI,
    #[default]
    Development,
    Debug,
}

/// Generate a default configuration
pub fn generate_default_config() -> ApicentricConfig {
    ApicentricConfig {
        ai: Some(AiConfig {
            provider: AiProviderKind::Gemini,
            model_path: None,
            api_key: Some("your-api-key-here".to_string()),
            model: Some("gemini-2.5-flash".to_string()),
        }),
        simulator: Some(crate::simulator::config::SimulatorConfig::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = ApicentricConfig::default();
        assert!(config.ai.is_none());
        assert!(config.simulator.is_none());
    }

    #[test]
    fn generate_default_config_works() {
        let config = generate_default_config();
        assert!(config.ai.is_some());
        assert!(config.simulator.is_some());
    }
}
