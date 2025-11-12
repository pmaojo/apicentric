use apicentric::cloud::handlers::{ApiResponse, UpdateConfigRequest, ValidateConfigResponse};
use apicentric::config::{ApicentricConfig, AiConfig, AiProviderKind};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_config_validation_valid() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create required directories
    std::fs::create_dir_all(temp_dir.path().join("routes")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("specs")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("services")).unwrap();
    
    // Create a valid configuration
    let config = ApicentricConfig::builder()
        .base_url("http://localhost:3000")
        .routes_dir(temp_dir.path().join("routes"))
        .specs_dir(temp_dir.path().join("specs"))
        .index_cache_path(temp_dir.path().join("index.json"))
        .simulator_services_dir(temp_dir.path().join("services"))
        .build()
        .unwrap();
    
    // Convert to JSON
    let config_json = serde_json::to_value(&config).unwrap();
    
    // Simulate validation request
    let request = UpdateConfigRequest {
        config: config_json,
    };
    
    // Parse and validate
    let parsed_config: ApicentricConfig = serde_json::from_value(request.config).unwrap();
    let validation_result = apicentric::validation::ConfigValidator::validate(&parsed_config);
    
    assert!(validation_result.is_ok(), "Valid configuration should pass validation");
}

#[tokio::test]
async fn test_config_validation_invalid_url() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create required directories
    std::fs::create_dir_all(temp_dir.path().join("routes")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("specs")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("services")).unwrap();
    
    // Create a configuration with invalid URL
    let mut config = ApicentricConfig::builder()
        .routes_dir(temp_dir.path().join("routes"))
        .specs_dir(temp_dir.path().join("specs"))
        .index_cache_path(temp_dir.path().join("index.json"))
        .simulator_services_dir(temp_dir.path().join("services"))
        .build()
        .unwrap();
    
    // Set invalid URL
    config.base_url = "not-a-valid-url".to_string();
    
    // Validate
    let validation_result = apicentric::validation::ConfigValidator::validate(&config);
    
    assert!(validation_result.is_err(), "Invalid URL should fail validation");
    let errors = validation_result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "base_url"));
}

#[tokio::test]
async fn test_config_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.json");
    
    // Create required directories
    std::fs::create_dir_all(temp_dir.path().join("routes")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("specs")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("services")).unwrap();
    
    // Create a configuration
    let config = ApicentricConfig::builder()
        .base_url("http://localhost:5000")
        .default_timeout(60000)
        .routes_dir(temp_dir.path().join("routes"))
        .specs_dir(temp_dir.path().join("specs"))
        .index_cache_path(temp_dir.path().join("index.json"))
        .simulator_services_dir(temp_dir.path().join("services"))
        .build()
        .unwrap();
    
    // Save the configuration
    apicentric::config::save_config(&config, &config_path).unwrap();
    
    // Load the configuration
    let loaded_config = apicentric::config::load_config(&config_path).unwrap();
    
    // Verify the loaded configuration matches
    assert_eq!(loaded_config.base_url, "http://localhost:5000");
    assert_eq!(loaded_config.default_timeout, 60000);
}

#[tokio::test]
async fn test_config_with_ai_settings() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create required directories
    std::fs::create_dir_all(temp_dir.path().join("routes")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("specs")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("services")).unwrap();
    
    // Create a configuration with AI settings
    let mut config = ApicentricConfig::builder()
        .routes_dir(temp_dir.path().join("routes"))
        .specs_dir(temp_dir.path().join("specs"))
        .index_cache_path(temp_dir.path().join("index.json"))
        .simulator_services_dir(temp_dir.path().join("services"))
        .build()
        .unwrap();
    
    config.ai = Some(AiConfig {
        provider: AiProviderKind::Openai,
        model_path: None,
        api_key: Some("test-key".to_string()),
        model: Some("gpt-4".to_string()),
    });
    
    // Validate
    let validation_result = apicentric::validation::ConfigValidator::validate(&config);
    assert!(validation_result.is_ok(), "Configuration with AI settings should be valid");
    
    // Convert to JSON and back
    let config_json = serde_json::to_value(&config).unwrap();
    let parsed_config: ApicentricConfig = serde_json::from_value(config_json).unwrap();
    
    assert!(parsed_config.ai.is_some());
    let ai_config = parsed_config.ai.unwrap();
    assert_eq!(ai_config.api_key, Some("test-key".to_string()));
    assert_eq!(ai_config.model, Some("gpt-4".to_string()));
}
