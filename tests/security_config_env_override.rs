#[cfg(test)]
mod tests {
    use apicentric::cloud::ai_handlers::ai_config_status;
    use apicentric::cloud::types::ApiResponse;
    use axum::response::Json;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ai_config_status_respects_env_var() {
        // Setup a temporary directory
        let dir = tempdir().unwrap();
        let custom_config_path = dir.path().join("custom_config.json");

        // Write a custom configuration file with UNIQUE values
        // Use "local" provider to distinguish from potential "openai" in root config
        let config_content = r#"{
            "ai": {
                "provider": "local",
                "model_path": "test-model.bin",
                "model": "llama-2"
            }
        }"#;
        fs::write(&custom_config_path, config_content).unwrap();

        // Set the environment variable to point to our custom config
        env::set_var("APICENTRIC_CONFIG_PATH", custom_config_path.to_str().unwrap());

        // Call the handler directly
        let result = ai_config_status().await;

        // Reset env var immediately
        env::remove_var("APICENTRIC_CONFIG_PATH");

        assert!(result.is_ok());
        let json_response: Json<ApiResponse<apicentric::cloud::ai_handlers::AiConfigResponse>> = result.unwrap();
        let response_body = json_response.0;

        assert!(response_body.success);
        let data = response_body.data.unwrap();

        // 🚨 REPRODUCTION CHECK:
        // If the bug exists, it will load "apicentric.json" from CWD (which has "openai").
        // We expect it to load "custom_config.json" (which has "local").

        assert_eq!(data.provider, Some("local".to_string()),
            "Failed to load custom config from APICENTRIC_CONFIG_PATH. Got provider: {:?}. Is the handler hardcoding 'apicentric.json'?",
            data.provider
        );
        assert_eq!(data.model, Some("llama-2".to_string()));
    }
}
