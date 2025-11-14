use apicentric::config::{ApicentricConfig, AiConfig, AiProviderKind, generate_default_config};
use apicentric::validation::ConfigValidator;
use tempfile::TempDir;

#[tokio::test]
async fn test_default_config_validation() {
    let config = ApicentricConfig::default();
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Default configuration should be valid");
}

#[tokio::test]
async fn test_config_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.json");
    
    // Create a configuration
    let config = generate_default_config();
    
    // Save the configuration
    apicentric::config::save_config(&config, &config_path).unwrap();
    
    // Load the configuration
    let loaded_config = apicentric::config::load_config(&config_path).unwrap();
    
    // Verify the loaded configuration has expected structure
    assert!(loaded_config.ai.is_some());
    assert!(loaded_config.simulator.is_some());
}

#[tokio::test]
async fn test_config_with_ai_settings() {
    let mut config = ApicentricConfig::default();
    
    config.ai = Some(AiConfig {
        provider: AiProviderKind::Openai,
        model_path: None,
        api_key: Some("test-key".to_string()),
        model: Some("gpt-4".to_string()),
    });
    
    // Validate
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Configuration with AI settings should be valid");
    
    // Convert to JSON and back
    let config_json = serde_json::to_value(&config).unwrap();
    let parsed_config: ApicentricConfig = serde_json::from_value(config_json).unwrap();
    
    assert!(parsed_config.ai.is_some());
    let ai_config = parsed_config.ai.unwrap();
    assert_eq!(ai_config.api_key, Some("test-key".to_string()));
    assert_eq!(ai_config.model, Some("gpt-4".to_string()));
}

#[tokio::test]
async fn test_ai_config_validation_missing_key() {
    let mut config = ApicentricConfig::default();
    
    config.ai = Some(AiConfig {
        provider: AiProviderKind::Openai,
        model_path: None,
        api_key: None, // Missing API key
        model: Some("gpt-4".to_string()),
    });
    
    // Validate
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Configuration with missing API key should fail validation");
    
    let errors = validation_result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "ai.api_key"));
}
