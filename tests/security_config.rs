#[cfg(test)]
mod tests {
    use apicentric::config::{AiConfig, AiProviderKind, ApicentricConfig};

    #[test]
    fn test_redact_sensitive_fields() {
        let mut config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("secret-key-123".to_string()),
                model: None,
            }),
            simulator: None,
        };

        config.redact_sensitive_fields();

        assert_eq!(
            config.ai.as_ref().unwrap().api_key,
            Some("********".to_string())
        );
    }

    #[test]
    fn test_merge_with_current_restores_secret() {
        let current_config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("secret-key-123".to_string()),
                model: None,
            }),
            simulator: None,
        };

        let mut new_config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("********".to_string()),
                model: Some("gpt-4".to_string()), // Changed field
            }),
            simulator: None,
        };

        new_config.merge_with_current(&current_config);

        assert_eq!(
            new_config.ai.as_ref().unwrap().api_key,
            Some("secret-key-123".to_string())
        );
        assert_eq!(
            new_config.ai.as_ref().unwrap().model,
            Some("gpt-4".to_string())
        );
    }

    #[test]
    fn test_merge_with_current_allows_update() {
        let current_config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("secret-key-123".to_string()),
                model: None,
            }),
            simulator: None,
        };

        let mut new_config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("new-secret-key".to_string()), // Changed key
                model: None,
            }),
            simulator: None,
        };

        new_config.merge_with_current(&current_config);

        assert_eq!(
            new_config.ai.as_ref().unwrap().api_key,
            Some("new-secret-key".to_string())
        );
    }

    #[test]
    fn test_merge_with_current_before_validation_succeeds() {
        use apicentric::validation::ConfigValidator;

        let current_config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("secret-key-123".to_string()),
                model: None,
            }),
            simulator: None,
        };

        let mut new_config = ApicentricConfig {
            ai: Some(AiConfig {
                provider: AiProviderKind::Openai,
                model_path: None,
                api_key: Some("********".to_string()),
                model: Some("gpt-4".to_string()),
            }),
            simulator: None,
        };

        // Note: Without merge_with_current, `new_config.validate()` might fail if it strictly checks
        // the API key format (though currently it just checks if empty, in the future it might check format).
        // By restoring the valid key first, we ensure validation passes properly.
        new_config.merge_with_current(&current_config);

        assert!(new_config.validate().is_ok());
        assert_eq!(
            new_config.ai.as_ref().unwrap().api_key,
            Some("secret-key-123".to_string())
        );
    }
}
