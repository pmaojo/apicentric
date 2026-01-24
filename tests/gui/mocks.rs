//! Mock Implementations for GUI Testing
//!
//! This module provides mock implementations of external dependencies
//! to enable isolated testing of GUI components.

#![cfg(feature = "gui")]

use apicentric::ai::AiProvider;
use apicentric::simulator::config::SimulatorConfig;
use apicentric::simulator::log::RequestLogEntry;
use apicentric::simulator::manager::ApiSimulatorManager;
use apicentric::{ApicentricError, ApicentricResult};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};

/// Mock AI Provider for testing
pub struct MockAiProvider {
    response: Arc<Mutex<Result<String, String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockAiProvider {
    /// Create a mock provider that returns successful YAML
    pub fn new_success(yaml_content: &str) -> Self {
        Self {
            response: Arc::new(Mutex::new(Ok(yaml_content.to_string()))),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a mock provider that returns an error
    pub fn new_error(error_message: &str) -> Self {
        Self {
            response: Arc::new(Mutex::new(Err(error_message.to_string()))),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the number of times generate_yaml was called
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Set a new response for subsequent calls
    pub fn set_response(&self, response: Result<String, String>) {
        *self.response.lock().unwrap() = response;
    }
}

#[async_trait]
impl AiProvider for MockAiProvider {
    async fn generate_yaml(&self, _prompt: &str) -> ApicentricResult<String> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        let response = self.response.lock().unwrap().clone();
        response.map_err(|e| ApicentricError::runtime_error(e, None::<String>))
    }
}

/// Create a valid test service YAML
pub fn create_test_service_yaml(name: &str) -> String {
    format!(
        r#"name: {}
version: "1.0"
description: "Test service"
server:
  port: 8080
  base_path: "/api"
endpoints:
  - method: GET
    path: "/test"
    responses:
      200:
        content_type: "application/json"
        body: '{{"status": "ok"}}'
"#,
        name
    )
}

/// Create a test request log entry
pub fn create_test_log_entry(method: &str, path: &str, status: u16) -> RequestLogEntry {
    RequestLogEntry {
        timestamp: Utc::now(),
        service: "test-service".to_string(),
        endpoint: Some(0),
        method: method.to_string(),
        path: path.to_string(),
        status,
        payload: None,
    }
}

/// Create a test simulator manager with default config
pub fn create_test_manager() -> Arc<ApiSimulatorManager> {
    let config = SimulatorConfig {
        enabled: true,
        services_dir: std::path::PathBuf::from("test_services"),
        port_range: apicentric::simulator::config::PortRange {
            start: 9000,
            end: 9099,
        },
        db_path: std::path::PathBuf::from(":memory:"),
        admin_port: None,
        global_behavior: None,
    };
    Arc::new(ApiSimulatorManager::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ai_provider_success() {
        let yaml = create_test_service_yaml("test");
        let provider = MockAiProvider::new_success(&yaml);

        let result = provider.generate_yaml("test prompt").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), yaml);
        assert_eq!(provider.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_ai_provider_error() {
        let provider = MockAiProvider::new_error("Test error");

        let result = provider.generate_yaml("test prompt").await;
        assert!(result.is_err());
        assert_eq!(provider.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_ai_provider_multiple_calls() {
        let provider = MockAiProvider::new_success("test");

        provider.generate_yaml("prompt 1").await.ok();
        provider.generate_yaml("prompt 2").await.ok();
        provider.generate_yaml("prompt 3").await.ok();

        assert_eq!(provider.call_count(), 3);
    }

    #[tokio::test]
    async fn test_mock_ai_provider_response_change() {
        let provider = MockAiProvider::new_success("first");

        let result1 = provider.generate_yaml("prompt").await;
        assert!(result1.is_ok());

        provider.set_response(Err("error".to_string()));

        let result2 = provider.generate_yaml("prompt").await;
        assert!(result2.is_err());
    }

    #[test]
    fn test_create_test_service_yaml() {
        let yaml = create_test_service_yaml("my-service");
        assert!(yaml.contains("name: my-service"));
        assert!(yaml.contains("version: \"1.0\""));
        assert!(yaml.contains("port: 8080"));
    }

    #[test]
    fn test_create_test_log_entry() {
        let entry = create_test_log_entry("GET", "/api/test", 200);
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.path, "/api/test");
        assert_eq!(entry.status, 200);
        assert_eq!(entry.service, "test-service");
    }

    #[test]
    fn test_create_test_manager() {
        let manager = create_test_manager();
        // Just verify it can be created without panicking
        assert!(Arc::strong_count(&manager) >= 1);
    }
}
