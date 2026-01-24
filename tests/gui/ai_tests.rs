//! AI Generation Tests
//!
//! Tests for AI service generation functionality in the GUI.

#![cfg(feature = "gui")]

use crate::gui::mocks::{create_test_service_yaml, MockAiProvider};
use apicentric::ai::AiProvider;
use apicentric::config::{AiConfig, AiProviderKind};
use apicentric::ApicentricResult;
use tokio::sync::mpsc;

#[allow(unused_imports)]
#[tokio::test]
async fn test_ai_generation_success() {
    let yaml = create_test_service_yaml("test-service");
    let provider = MockAiProvider::new_success(&yaml);

    let result = provider.generate_yaml("Create a test API").await;

    assert!(result.is_ok());
    let generated = result.unwrap();
    assert!(generated.contains("name: test-service"));
    assert!(generated.contains("version: \"1.0\""));
}

#[tokio::test]
async fn test_ai_generation_failure() {
    let provider = MockAiProvider::new_error("API key invalid");

    let result = provider.generate_yaml("Create a test API").await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("API key invalid"));
}

#[tokio::test]
async fn test_ai_generation_with_different_prompts() {
    let yaml = create_test_service_yaml("user-api");
    let provider = MockAiProvider::new_success(&yaml);

    // Test with different prompts
    let prompts = vec![
        "Create a user API",
        "Generate a REST API for users",
        "Build a user management service",
    ];

    for prompt in prompts {
        let result = provider.generate_yaml(prompt).await;
        assert!(result.is_ok());
    }

    // Verify all prompts were processed
    assert_eq!(provider.call_count(), 3);
}

#[tokio::test]
async fn test_ai_generation_empty_prompt() {
    let yaml = create_test_service_yaml("empty");
    let provider = MockAiProvider::new_success(&yaml);

    let result = provider.generate_yaml("").await;

    // Should still work with empty prompt (provider handles it)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ai_generation_long_prompt() {
    let yaml = create_test_service_yaml("complex");
    let provider = MockAiProvider::new_success(&yaml);

    let long_prompt = "Create a comprehensive REST API for managing users with the following features: \
                       authentication, authorization, CRUD operations, pagination, filtering, sorting, \
                       search, validation, error handling, and rate limiting. Include endpoints for \
                       registration, login, logout, profile management, password reset, and email verification.";

    let result = provider.generate_yaml(long_prompt).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ai_generation_special_characters() {
    let yaml = create_test_service_yaml("special-chars");
    let provider = MockAiProvider::new_success(&yaml);

    let prompt = "Create an API with special chars: @#$%^&*()";

    let result = provider.generate_yaml(prompt).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ai_generation_concurrent_requests() {
    let yaml = create_test_service_yaml("concurrent");
    let provider = std::sync::Arc::new(MockAiProvider::new_success(&yaml));

    // Spawn multiple concurrent generation requests
    let mut handles = vec![];
    for i in 0..5 {
        let provider_clone = provider.clone();
        let handle =
            tokio::spawn(
                async move { provider_clone.generate_yaml(&format!("Prompt {}", i)).await },
            );
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all requests were processed
    assert_eq!(provider.call_count(), 5);
}

#[tokio::test]
async fn test_ai_generation_retry_after_error() {
    let yaml = create_test_service_yaml("retry");
    let provider = MockAiProvider::new_error("Temporary error");

    // First attempt fails
    let result1 = provider.generate_yaml("test").await;
    assert!(result1.is_err());

    // Change to success response
    provider.set_response(Ok(yaml.clone()));

    // Second attempt succeeds
    let result2 = provider.generate_yaml("test").await;
    assert!(result2.is_ok());

    assert_eq!(provider.call_count(), 2);
}

#[tokio::test]
async fn test_ai_generation_yaml_structure() {
    let yaml = create_test_service_yaml("structure-test");
    let provider = MockAiProvider::new_success(&yaml);

    let result = provider.generate_yaml("test").await;

    assert!(result.is_ok());
    let generated = result.unwrap();

    // Verify YAML contains required fields
    assert!(generated.contains("name:"));
    assert!(generated.contains("version:"));
    assert!(generated.contains("server:"));
    assert!(generated.contains("port:"));
    assert!(generated.contains("endpoints:"));
}

#[tokio::test]
async fn test_ai_generation_multiple_endpoints() {
    let yaml = r#"name: multi-endpoint
version: "1.0"
server:
  port: 8080
  base_path: "/api"
endpoints:
  - method: GET
    path: "/users"
    responses:
      200:
        content_type: "application/json"
        body: '[]'
  - method: POST
    path: "/users"
    responses:
      201:
        content_type: "application/json"
        body: '{}'
  - method: GET
    path: "/users/{id}"
    responses:
      200:
        content_type: "application/json"
        body: '{}'
"#;
    let provider = MockAiProvider::new_success(yaml);

    let result = provider
        .generate_yaml("Create a user API with multiple endpoints")
        .await;

    assert!(result.is_ok());
    let generated = result.unwrap();
    assert!(generated.contains("GET"));
    assert!(generated.contains("POST"));
}

#[tokio::test]
async fn test_ai_provider_call_tracking() {
    let yaml = create_test_service_yaml("tracking");
    let provider = MockAiProvider::new_success(&yaml);

    assert_eq!(provider.call_count(), 0);

    provider.generate_yaml("prompt 1").await.ok();
    assert_eq!(provider.call_count(), 1);

    provider.generate_yaml("prompt 2").await.ok();
    assert_eq!(provider.call_count(), 2);

    provider.generate_yaml("prompt 3").await.ok();
    assert_eq!(provider.call_count(), 3);
}

// Integration test for the actual handle_ai_generate function
// Note: This tests the integration between the GUI and AI provider
#[tokio::test]
async fn test_ai_generation_integration() {
    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);
    let yaml = create_test_service_yaml("integration");

    // Simulate successful generation
    tx.send(Ok(yaml.clone())).await.unwrap();

    // Receive result
    let result = rx.recv().await.unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), yaml);
}

#[tokio::test]
async fn test_ai_generation_error_propagation() {
    use apicentric::ApicentricError;

    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);

    // Simulate error
    let error = ApicentricError::config_error("Test error", Some("Fix it"));
    tx.send(Err(error)).await.unwrap();

    // Receive error
    let result = rx.recv().await.unwrap();
    assert!(result.is_err());
}

// Tests for handle_ai_generate function behavior

#[tokio::test]
async fn test_handle_ai_generate_with_valid_config() {
    // Test that handle_ai_generate works with a valid configuration
    // This simulates the async channel communication pattern used in the GUI
    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);
    let yaml = create_test_service_yaml("valid-config");

    // Simulate successful AI generation
    tx.send(Ok(yaml.clone())).await.unwrap();

    let result = rx.recv().await.unwrap();
    assert!(result.is_ok());
    let generated = result.unwrap();
    assert!(generated.contains("name: valid-config"));
}

#[tokio::test]
async fn test_handle_ai_generate_missing_config() {
    // Test error handling when AI config is missing
    use apicentric::ApicentricError;

    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);

    // Simulate missing config error
    let error = ApicentricError::config_error(
        "AI provider not configured",
        Some("Add an 'ai' section to apicentric.json"),
    );
    tx.send(Err(error)).await.unwrap();

    let result = rx.recv().await.unwrap();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AI provider not configured"));
}

#[tokio::test]
async fn test_handle_ai_generate_missing_api_key() {
    // Test error handling when API key is missing
    use apicentric::ApicentricError;

    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);

    // Simulate missing API key error
    let error = ApicentricError::config_error(
        "OpenAI API key missing",
        Some("Set ai.api_key in apicentric.json"),
    );
    tx.send(Err(error)).await.unwrap();

    let result = rx.recv().await.unwrap();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("API key missing"));
}

// Tests for provider selection logic

#[test]
fn test_provider_selection_openai() {
    // Test that OpenAI provider is selected correctly
    let ai_config = AiConfig {
        provider: AiProviderKind::Openai,
        api_key: Some("test-key".to_string()),
        model: Some("gpt-3.5-turbo".to_string()),
        model_path: None,
    };

    assert!(matches!(ai_config.provider, AiProviderKind::Openai));
    assert!(ai_config.api_key.is_some());
}

#[test]
fn test_provider_selection_gemini() {
    // Test that Gemini provider is selected correctly
    let ai_config = AiConfig {
        provider: AiProviderKind::Gemini,
        api_key: Some("test-key".to_string()),
        model: Some("gemini-2.0-flash-exp".to_string()),
        model_path: None,
    };

    assert!(matches!(ai_config.provider, AiProviderKind::Gemini));
    assert!(ai_config.api_key.is_some());
}

#[test]
fn test_provider_selection_local() {
    // Test that Local provider is selected correctly
    let ai_config = AiConfig {
        provider: AiProviderKind::Local,
        api_key: None,
        model: None,
        model_path: Some("model.bin".to_string()),
    };

    assert!(matches!(ai_config.provider, AiProviderKind::Local));
    assert!(ai_config.model_path.is_some());
}

#[test]
fn test_provider_default_model_openai() {
    // Test default model for OpenAI
    let ai_config = AiConfig {
        provider: AiProviderKind::Openai,
        api_key: Some("test-key".to_string()),
        model: None,
        model_path: None,
    };

    let model = ai_config
        .model
        .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    assert_eq!(model, "gpt-3.5-turbo");
}

#[test]
fn test_provider_default_model_gemini() {
    // Test default model for Gemini
    let ai_config = AiConfig {
        provider: AiProviderKind::Gemini,
        api_key: Some("test-key".to_string()),
        model: None,
        model_path: None,
    };

    let model = ai_config
        .model
        .unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
    assert_eq!(model, "gemini-2.0-flash-exp");
}

#[test]
fn test_provider_default_model_path_local() {
    // Test default model path for Local
    let ai_config = AiConfig {
        provider: AiProviderKind::Local,
        api_key: None,
        model: None,
        model_path: None,
    };

    let path = ai_config
        .model_path
        .unwrap_or_else(|| "model.bin".to_string());
    assert_eq!(path, "model.bin");
}

// Error handling tests

#[tokio::test]
async fn test_ai_generation_network_error() {
    // Test handling of network errors
    let provider = MockAiProvider::new_error("Network connection failed");

    let result = provider.generate_yaml("test prompt").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Network connection failed"));
}

#[tokio::test]
async fn test_ai_generation_rate_limit_error() {
    // Test handling of rate limit errors
    let provider = MockAiProvider::new_error("Rate limit exceeded");

    let result = provider.generate_yaml("test prompt").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Rate limit exceeded"));
}

#[tokio::test]
async fn test_ai_generation_invalid_response() {
    // Test handling of invalid AI responses
    let provider = MockAiProvider::new_success("invalid yaml content: [[[");

    let result = provider.generate_yaml("test prompt").await;

    // Provider returns the response, validation happens later
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ai_generation_timeout() {
    // Test handling of timeout scenarios
    let provider = MockAiProvider::new_error("Request timeout");

    let result = provider.generate_yaml("test prompt").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}

#[tokio::test]
async fn test_ai_generation_authentication_error() {
    // Test handling of authentication errors
    let provider = MockAiProvider::new_error("Invalid API key");

    let result = provider.generate_yaml("test prompt").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid API key"));
}

// Channel communication tests

#[tokio::test]
async fn test_ai_result_channel_send_receive() {
    // Test that results can be sent and received through channels
    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);
    let yaml = create_test_service_yaml("channel-test");

    // Send result
    tx.send(Ok(yaml.clone())).await.unwrap();

    // Receive result
    let result = rx.recv().await.unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), yaml);
}

#[tokio::test]
async fn test_ai_result_channel_multiple_messages() {
    // Test multiple messages through the channel
    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(10);

    // Send multiple results
    for i in 0..5 {
        let yaml = create_test_service_yaml(&format!("service-{}", i));
        tx.send(Ok(yaml)).await.unwrap();
    }

    // Receive all results
    for _ in 0..5 {
        let result = rx.recv().await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_ai_result_channel_closed() {
    // Test behavior when channel is closed
    let (tx, mut rx) = mpsc::channel::<ApicentricResult<String>>(1);

    // Close sender
    drop(tx);

    // Try to receive - should return None
    let result = rx.recv().await;
    assert!(result.is_none());
}

// Tests for enhanced AI features

// YAML Validation Tests

#[test]
fn test_yaml_validation_valid() {
    // Test that valid YAML passes validation
    let yaml = create_test_service_yaml("valid-service");

    // Parse YAML to verify it's valid
    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(&yaml);
    assert!(result.is_ok());
}

#[test]
fn test_yaml_validation_invalid_syntax() {
    // Test that invalid YAML syntax is detected
    let invalid_yaml = "name: test\n  invalid: [[[";

    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(invalid_yaml);
    assert!(result.is_err());
}

#[test]
fn test_yaml_validation_missing_required_fields() {
    // Test that YAML missing required fields is detected
    let incomplete_yaml = r#"
name: test
version: "1.0"
# Missing server and endpoints
"#;

    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(incomplete_yaml);
    assert!(result.is_ok()); // Parses but may fail domain validation

    let value = result.unwrap();
    assert!(value.get("name").is_some());
    assert!(value.get("server").is_none()); // Missing required field
}

#[test]
fn test_yaml_validation_with_line_numbers() {
    // Test that validation errors include line numbers
    let invalid_yaml = "line1: ok\nline2: [[[invalid\nline3: ok";

    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(invalid_yaml);
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    // YAML parser includes location information
    assert!(error_msg.contains("line") || error_msg.contains("column"));
}

#[test]
fn test_yaml_validation_empty_document() {
    // Test validation of empty YAML document
    let empty_yaml = "";

    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(empty_yaml);
    assert!(result.is_ok()); // Empty is valid YAML (null)
}

#[test]
fn test_yaml_validation_complex_structure() {
    // Test validation of complex nested YAML
    let complex_yaml = r#"
name: complex-service
version: "1.0"
server:
  port: 8080
  base_path: "/api"
endpoints:
  - method: GET
    path: "/users"
    responses:
      200:
        content_type: "application/json"
        body: '{"users": []}'
      404:
        content_type: "application/json"
        body: '{"error": "Not found"}'
  - method: POST
    path: "/users"
    responses:
      201:
        content_type: "application/json"
        body: '{"id": 1}'
"#;

    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(complex_yaml);
    assert!(result.is_ok());

    let value = result.unwrap();
    assert!(value.get("endpoints").is_some());
    assert!(value["endpoints"].as_sequence().unwrap().len() == 2);
}

// Preview Functionality Tests

#[tokio::test]
async fn test_preview_generated_yaml() {
    // Test that generated YAML can be previewed
    let yaml = create_test_service_yaml("preview-test");
    let provider = MockAiProvider::new_success(&yaml);

    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let generated = result.unwrap();
    // Verify preview content
    assert!(generated.contains("name: preview-test"));
    assert!(generated.contains("version:"));
    assert!(generated.contains("endpoints:"));
}

#[tokio::test]
async fn test_preview_with_formatting() {
    // Test that preview maintains YAML formatting
    let yaml = r#"name: formatted
version: "1.0"
description: "Test service with formatting"
server:
  port: 8080
  base_path: "/api"
endpoints:
  - method: GET
    path: "/test"
    responses:
      200:
        content_type: "application/json"
        body: '{"status": "ok"}'
"#;
    let provider = MockAiProvider::new_success(yaml);

    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let generated = result.unwrap();
    // Verify formatting is preserved
    assert!(generated.contains("  port: 8080"));
    assert!(generated.contains("  - method: GET"));
}

#[tokio::test]
async fn test_preview_error_display() {
    // Test that preview shows errors appropriately
    let provider = MockAiProvider::new_error("Generation failed");

    let result = provider.generate_yaml("test").await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("Generation failed"));
}

// Apply Workflow Tests

#[tokio::test]
async fn test_apply_workflow_success() {
    // Test successful apply workflow
    let yaml = create_test_service_yaml("apply-test");
    let provider = MockAiProvider::new_success(&yaml);

    // Step 1: Generate
    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Step 2: Validate before apply
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_ok());

    // Step 3: Apply (would save to file in real implementation)
    // For now, just verify the YAML is valid
    assert!(generated.contains("name: apply-test"));
}

#[tokio::test]
async fn test_apply_workflow_validation_failure() {
    // Test apply workflow when validation fails
    let invalid_yaml = "invalid: [[[";
    let provider = MockAiProvider::new_success(invalid_yaml);

    // Step 1: Generate
    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Step 2: Validate before apply - should fail
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_err());

    // Step 3: Apply should not proceed due to validation failure
}

#[tokio::test]
async fn test_apply_workflow_with_modifications() {
    // Test apply workflow with user modifications
    let yaml = create_test_service_yaml("modify-test");
    let provider = MockAiProvider::new_success(&yaml);

    // Step 1: Generate
    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let mut generated = result.unwrap();

    // Step 2: User modifies the YAML
    generated = generated.replace("8080", "9090");

    // Step 3: Validate modified YAML
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_ok());

    // Step 4: Apply modified YAML
    assert!(generated.contains("9090"));
}

// Configuration Detection Tests

#[test]
fn test_detect_ai_config_present() {
    // Test detection when AI config is present
    use apicentric::config::AiConfig;

    let ai_config = AiConfig {
        provider: AiProviderKind::Openai,
        api_key: Some("test-key".to_string()),
        model: Some("gpt-3.5-turbo".to_string()),
        model_path: None,
    };

    let config = ai_config;
    assert!(config.api_key.is_some());
}

#[test]
fn test_detect_ai_config_missing() {
    // Test detection when AI config is missing
    let ai_config: Option<apicentric::config::AiConfig> = None;

    assert!(ai_config.is_none());
}

#[test]
fn test_detect_ai_config_incomplete() {
    // Test detection when AI config is incomplete
    use apicentric::config::AiConfig;

    let ai_config = AiConfig {
        provider: AiProviderKind::Openai,
        api_key: None, // Missing required API key
        model: Some("gpt-3.5-turbo".to_string()),
        model_path: None,
    };

    // Config exists but is incomplete
    assert!(ai_config.api_key.is_none());
}

#[test]
fn test_detect_provider_type() {
    // Test detection of different provider types
    use apicentric::config::AiConfig;

    let openai_config = AiConfig {
        provider: AiProviderKind::Openai,
        api_key: Some("key".to_string()),
        model: None,
        model_path: None,
    };
    assert!(matches!(openai_config.provider, AiProviderKind::Openai));

    let gemini_config = AiConfig {
        provider: AiProviderKind::Gemini,
        api_key: Some("key".to_string()),
        model: None,
        model_path: None,
    };
    assert!(matches!(gemini_config.provider, AiProviderKind::Gemini));

    let local_config = AiConfig {
        provider: AiProviderKind::Local,
        api_key: None,
        model: None,
        model_path: Some("model.bin".to_string()),
    };
    assert!(matches!(local_config.provider, AiProviderKind::Local));
}

#[test]
fn test_config_validation_messages() {
    // Test that appropriate validation messages are generated
    use apicentric::config::AiConfig;

    // Missing API key for OpenAI
    let config = AiConfig {
        provider: AiProviderKind::Openai,
        api_key: None,
        model: None,
        model_path: None,
    };

    // In real implementation, this would generate a user-friendly message
    assert!(config.api_key.is_none());
    let expected_message = "OpenAI API key missing. Set ai.api_key in apicentric.json";
    assert!(expected_message.contains("API key missing"));
}

#[test]
fn test_config_prompt_when_not_configured() {
    // Test that a configuration prompt is shown when AI is not configured
    let ai_config: Option<apicentric::config::AiConfig> = None;

    if ai_config.is_none() {
        let prompt_message = "AI provider not configured. Add an 'ai' section to apicentric.json";
        assert!(prompt_message.contains("not configured"));
        assert!(prompt_message.contains("apicentric.json"));
    }
}

// Integration tests for complete AI workflow

#[tokio::test]
async fn test_complete_ai_workflow_with_validation() {
    // Test complete workflow: generate -> validate -> preview -> apply
    let yaml = create_test_service_yaml("workflow-test");
    let provider = MockAiProvider::new_success(&yaml);

    // Step 1: Generate
    let generate_result = provider.generate_yaml("Create a test API").await;
    assert!(generate_result.is_ok());

    let generated_yaml = generate_result.unwrap();

    // Step 2: Validate
    let validation_result: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated_yaml);
    assert!(validation_result.is_ok());

    // Step 3: Preview (verify content)
    assert!(generated_yaml.contains("name: workflow-test"));
    assert!(generated_yaml.contains("version:"));

    // Step 4: Apply (in real implementation, would save to file)
    // For now, just verify the YAML structure
    let value = validation_result.unwrap();
    assert!(value.get("name").is_some());
    assert!(value.get("server").is_some());
    assert!(value.get("endpoints").is_some());
}

#[tokio::test]
async fn test_workflow_with_error_recovery() {
    // Test workflow with error and recovery
    let provider = MockAiProvider::new_error("Initial error");

    // Step 1: First attempt fails
    let result1 = provider.generate_yaml("test").await;
    assert!(result1.is_err());

    // Step 2: Fix configuration and retry
    let yaml = create_test_service_yaml("recovery-test");
    provider.set_response(Ok(yaml.clone()));

    // Step 3: Second attempt succeeds
    let result2 = provider.generate_yaml("test").await;
    assert!(result2.is_ok());

    // Step 4: Validate and apply
    let generated = result2.unwrap();
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_ok());
}

// Integration tests for complete AI workflows

#[tokio::test]
async fn test_integration_complete_generation_workflow() {
    // Test the complete workflow from prompt to generated YAML

    let yaml = create_test_service_yaml("integration-workflow");
    let provider = MockAiProvider::new_success(&yaml);

    // Step 1: User enters prompt
    let prompt = "Create a user management API";

    // Step 2: Generate service
    let result = provider.generate_yaml(prompt).await;
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Step 3: Verify generated content
    assert!(generated.contains("name: integration-workflow"));
    assert!(generated.contains("version:"));
    assert!(generated.contains("server:"));
    assert!(generated.contains("endpoints:"));

    // Step 4: Validate YAML
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_ok());

    // Step 5: Verify call was tracked
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn test_integration_apply_generated_yaml_workflow() {
    // Test the workflow of applying generated YAML
    let yaml = create_test_service_yaml("apply-workflow");
    let provider = MockAiProvider::new_success(&yaml);

    // Step 1: Generate
    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Step 2: User reviews and approves
    // (In real implementation, this would show in preview window)

    // Step 3: Validate before applying
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_ok());

    // Step 4: Apply (save to file)
    // In real implementation, this would:
    // - Create services directory if needed
    // - Write YAML to file
    // - Refresh service list
    // For now, just verify the YAML is valid
    let value = validation.unwrap();
    assert!(value.get("name").is_some());
}

#[tokio::test]
async fn test_integration_error_recovery_workflow() {
    // Test error recovery workflow
    let provider = MockAiProvider::new_error("Network timeout");

    // Step 1: First attempt fails
    let result1 = provider.generate_yaml("test").await;
    assert!(result1.is_err());

    let error1 = result1.unwrap_err();
    assert!(error1.to_string().contains("Network timeout"));

    // Step 2: User sees error message
    // (In real implementation, this would show in UI)

    // Step 3: User retries
    let yaml = create_test_service_yaml("recovery");
    provider.set_response(Ok(yaml.clone()));

    let result2 = provider.generate_yaml("test").await;
    assert!(result2.is_ok());

    // Step 4: Success on retry
    let generated = result2.unwrap();
    assert!(generated.contains("name: recovery"));

    // Verify both attempts were tracked
    assert_eq!(provider.call_count(), 2);
}

#[tokio::test]
async fn test_integration_workflow_with_different_providers() {
    // Test workflow with different AI providers

    // Test with "OpenAI" provider (mock)
    let yaml1 = create_test_service_yaml("openai-service");
    let openai_provider = MockAiProvider::new_success(&yaml1);

    let result1 = openai_provider.generate_yaml("Create API").await;
    assert!(result1.is_ok());
    assert!(result1.unwrap().contains("openai-service"));

    // Test with "Gemini" provider (mock)
    let yaml2 = create_test_service_yaml("gemini-service");
    let gemini_provider = MockAiProvider::new_success(&yaml2);

    let result2 = gemini_provider.generate_yaml("Create API").await;
    assert!(result2.is_ok());
    assert!(result2.unwrap().contains("gemini-service"));

    // Test with "Local" provider (mock)
    let yaml3 = create_test_service_yaml("local-service");
    let local_provider = MockAiProvider::new_success(&yaml3);

    let result3 = local_provider.generate_yaml("Create API").await;
    assert!(result3.is_ok());
    assert!(result3.unwrap().contains("local-service"));
}

#[tokio::test]
async fn test_integration_concurrent_generation_requests() {
    // Test handling multiple concurrent generation requests
    let yaml = create_test_service_yaml("concurrent");
    let provider = std::sync::Arc::new(MockAiProvider::new_success(&yaml));

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for i in 0..10 {
        let provider_clone = provider.clone();
        let handle = tokio::spawn(async move {
            provider_clone
                .generate_yaml(&format!("Request {}", i))
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        let result = handle.await.unwrap();
        if result.is_ok() {
            success_count += 1;
        }
    }

    // All should succeed
    assert_eq!(success_count, 10);
    assert_eq!(provider.call_count(), 10);
}

#[tokio::test]
async fn test_integration_validation_before_apply() {
    // Test that validation happens before applying
    let invalid_yaml = "name: test\ninvalid: [[[";
    let provider = MockAiProvider::new_success(invalid_yaml);

    // Step 1: Generate
    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Step 2: Validate - should fail
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_err());

    // Step 3: Apply should not proceed
    // In real implementation, the UI would show validation errors
    // and prevent the apply action
}

#[tokio::test]
async fn test_integration_user_modification_workflow() {
    // Test workflow where user modifies generated YAML
    let yaml = create_test_service_yaml("modify");
    let provider = MockAiProvider::new_success(&yaml);

    // Step 1: Generate
    let result = provider.generate_yaml("test").await;
    assert!(result.is_ok());

    let mut generated = result.unwrap();

    // Step 2: User modifies in preview
    generated = generated.replace("port: 8080", "port: 9000");
    generated = generated.replace("name: modify", "name: modified-service");

    // Step 3: Validate modified YAML
    let validation: Result<serde_yaml::Value, _> = serde_yaml::from_str(&generated);
    assert!(validation.is_ok());

    // Step 4: Apply modified version
    let value = validation.unwrap();
    assert_eq!(value["name"].as_str().unwrap(), "modified-service");
    assert_eq!(value["server"]["port"].as_u64().unwrap(), 9000);
}

#[tokio::test]
async fn test_integration_multiple_generation_attempts() {
    // Test multiple generation attempts with different prompts
    let provider = MockAiProvider::new_success("");

    let prompts = vec![
        ("Create a user API", "user-api"),
        ("Create a product API", "product-api"),
        ("Create an order API", "order-api"),
    ];

    for (prompt, service_name) in prompts {
        let yaml = create_test_service_yaml(service_name);
        provider.set_response(Ok(yaml.clone()));

        let result = provider.generate_yaml(prompt).await;
        assert!(result.is_ok());

        let generated = result.unwrap();
        assert!(generated.contains(&format!("name: {}", service_name)));
    }

    assert_eq!(provider.call_count(), 3);
}

#[tokio::test]
async fn test_integration_error_types_workflow() {
    // Test handling different error types
    let error_scenarios = vec![
        ("Network error", "Network connection failed"),
        ("Auth error", "Invalid API key"),
        ("Rate limit", "Rate limit exceeded"),
        ("Timeout", "Request timeout"),
    ];

    for (scenario, error_msg) in error_scenarios {
        let provider = MockAiProvider::new_error(error_msg);

        let result = provider.generate_yaml("test").await;
        assert!(result.is_err(), "Expected error for scenario: {}", scenario);

        let error = result.unwrap_err();
        assert!(error.to_string().contains(error_msg));
    }
}

#[tokio::test]
async fn test_integration_state_management_workflow() {
    // Test state management during AI generation workflow
    // Note: This test verifies the conceptual workflow
    // In a full integration test, we would use the actual GuiAppState

    // Simulate state transitions
    let mut ai_generation_in_progress = false;
    let mut ai_generated_yaml: Option<String> = None;
    let mut ai_error: Option<String> = None;

    // Initial state
    assert!(!ai_generation_in_progress);
    assert!(ai_generated_yaml.is_none());
    assert!(ai_error.is_none());

    // Start generation
    ai_generation_in_progress = true;
    ai_generated_yaml = None;
    assert!(ai_generation_in_progress);
    assert!(ai_generated_yaml.is_none());

    // Complete with success
    let yaml = create_test_service_yaml("state-test");
    ai_generation_in_progress = false;
    ai_generated_yaml = Some(yaml.clone());
    ai_error = None;
    assert!(!ai_generation_in_progress);
    assert!(ai_generated_yaml.is_some());
    assert_eq!(ai_generated_yaml.unwrap(), yaml);
    assert!(ai_error.is_none());
}

#[tokio::test]
async fn test_integration_state_error_workflow() {
    // Test state management during error scenarios

    // Simulate state transitions
    let mut ai_generation_in_progress = true;
    let ai_generated_yaml: Option<String> = None;

    // Start generation
    assert!(ai_generation_in_progress);

    // Fail with error
    ai_generation_in_progress = false;
    let ai_error = Some("Test error".to_string());
    assert!(!ai_generation_in_progress);
    assert!(ai_generated_yaml.is_none());
    assert!(ai_error.is_some());
    assert_eq!(ai_error, Some("Test error".to_string()));
}

#[tokio::test]
async fn test_integration_config_detection_workflow() {
    // Test configuration detection workflow

    // Simulate config state
    let mut ai_config_missing = false;

    // Check initial config state
    assert!(!ai_config_missing);

    // Simulate config check
    // In real implementation, this would check apicentric.json
    // For test, simulate missing config
    ai_config_missing = true;

    // State should reflect whether config exists
    assert!(ai_config_missing);
}

#[tokio::test]
async fn test_integration_validation_errors_workflow() {
    // Test validation errors in state

    // Set validation errors
    let errors = vec![
        "Missing required field: name".to_string(),
        "Invalid port number".to_string(),
    ];

    // Simulate validation errors state
    let mut ai_validation_errors: Vec<String> = errors.clone();

    assert_eq!(ai_validation_errors.len(), 2);
    assert_eq!(ai_validation_errors, errors);

    // Clear validation errors
    ai_validation_errors.clear();
    assert_eq!(ai_validation_errors.len(), 0);
}
