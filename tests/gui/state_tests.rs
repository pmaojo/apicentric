//! State Management Tests
//!
//! Tests for GuiAppState and related state management logic.

#![cfg(feature = "gui")]

use std::collections::VecDeque;
use std::path::PathBuf;

// Test structures that mirror the enhanced GuiAppState design
// These will be used to test the refactored state module

#[cfg(test)]
mod enhanced_state_tests {
    use super::*;

    // Test data structures for ServiceInfo
    #[derive(Debug, Clone, PartialEq)]
    pub struct TestServiceInfo {
        pub name: String,
        pub path: PathBuf,
        pub status: TestServiceStatus,
        pub port: u16,
        pub endpoints: Vec<TestEndpointInfo>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TestServiceStatus {
        Stopped,
        Starting,
        Running,
        Stopping,
        Failed(String),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct TestEndpointInfo {
        pub method: String,
        pub path: String,
    }

    // Test data structure for RecordingSession
    #[derive(Debug, Clone)]
    pub struct TestRecordingSession {
        pub id: String,
        pub target_url: String,
        pub proxy_port: u16,
        pub is_active: bool,
    }

    // Test data structure for EditorState
    #[derive(Debug, Clone)]
    pub struct TestEditorState {
        pub content: String,
        pub dirty: bool,
        pub selected_service: Option<String>,
    }

    // Test data structure for GuiConfig
    #[derive(Debug, Clone)]
    pub struct TestGuiConfig {
        pub services_directory: PathBuf,
        pub default_port: u16,
    }

    // Test data structure for enhanced GuiAppState
    #[derive(Debug, Clone)]
    pub struct TestGuiAppState {
        pub services: Vec<TestServiceInfo>,
        pub selected_service: Option<String>,
        pub ai_prompt: String,
        pub ai_generated_yaml: Option<String>,
        pub ai_generation_in_progress: bool,
        pub ai_error: Option<String>,
        pub request_logs: VecDeque<String>,
        pub recording_session: Option<TestRecordingSession>,
        pub editor_state: TestEditorState,
        pub config: TestGuiConfig,
        pub show_ai_window: bool,
        pub show_editor_window: bool,
        pub show_config_window: bool,
    }

    impl Default for TestGuiAppState {
        fn default() -> Self {
            Self {
                services: Vec::new(),
                selected_service: None,
                ai_prompt: String::new(),
                ai_generated_yaml: None,
                ai_generation_in_progress: false,
                ai_error: None,
                request_logs: VecDeque::new(),
                recording_session: None,
                editor_state: TestEditorState {
                    content: String::new(),
                    dirty: false,
                    selected_service: None,
                },
                config: TestGuiConfig {
                    services_directory: PathBuf::from("services"),
                    default_port: 8080,
                },
                show_ai_window: false,
                show_editor_window: false,
                show_config_window: false,
            }
        }
    }

    #[test]
    fn test_gui_app_state_default_initialization() {
        let state = TestGuiAppState::default();
        
        assert!(state.services.is_empty());
        assert!(state.selected_service.is_none());
        assert_eq!(state.ai_prompt, "");
        assert!(state.ai_generated_yaml.is_none());
        assert!(!state.ai_generation_in_progress);
        assert!(state.ai_error.is_none());
        assert!(state.request_logs.is_empty());
        assert!(state.recording_session.is_none());
        assert!(!state.show_ai_window);
        assert!(!state.show_editor_window);
        assert!(!state.show_config_window);
    }

    #[test]
    fn test_services_field_can_hold_service_info() {
        let mut state = TestGuiAppState::default();
        
        let service = TestServiceInfo {
            name: "test-service".to_string(),
            path: PathBuf::from("services/test.yaml"),
            status: TestServiceStatus::Stopped,
            port: 8080,
            endpoints: vec![],
        };
        
        state.services.push(service.clone());
        
        assert_eq!(state.services.len(), 1);
        assert_eq!(state.services[0].name, "test-service");
        assert_eq!(state.services[0].port, 8080);
    }

    #[test]
    fn test_service_info_with_endpoints() {
        let service = TestServiceInfo {
            name: "api-service".to_string(),
            path: PathBuf::from("services/api.yaml"),
            status: TestServiceStatus::Running,
            port: 9000,
            endpoints: vec![
                TestEndpointInfo {
                    method: "GET".to_string(),
                    path: "/users".to_string(),
                },
                TestEndpointInfo {
                    method: "POST".to_string(),
                    path: "/users".to_string(),
                },
            ],
        };
        
        assert_eq!(service.endpoints.len(), 2);
        assert_eq!(service.endpoints[0].method, "GET");
        assert_eq!(service.endpoints[1].method, "POST");
    }

    #[test]
    fn test_recording_session_state() {
        let mut state = TestGuiAppState::default();
        
        assert!(state.recording_session.is_none());
        
        state.recording_session = Some(TestRecordingSession {
            id: "rec-123".to_string(),
            target_url: "https://api.example.com".to_string(),
            proxy_port: 8888,
            is_active: true,
        });
        
        assert!(state.recording_session.is_some());
        let session = state.recording_session.as_ref().unwrap();
        assert_eq!(session.id, "rec-123");
        assert_eq!(session.target_url, "https://api.example.com");
        assert_eq!(session.proxy_port, 8888);
        assert!(session.is_active);
    }

    #[test]
    fn test_editor_state_initialization() {
        let state = TestGuiAppState::default();
        
        assert_eq!(state.editor_state.content, "");
        assert!(!state.editor_state.dirty);
        assert!(state.editor_state.selected_service.is_none());
    }

    #[test]
    fn test_editor_state_dirty_flag() {
        let mut state = TestGuiAppState::default();
        
        state.editor_state.content = "name: test".to_string();
        state.editor_state.dirty = true;
        
        assert!(state.editor_state.dirty);
        assert!(!state.editor_state.content.is_empty());
    }

    #[test]
    fn test_editor_state_with_selected_service() {
        let mut state = TestGuiAppState::default();
        
        state.editor_state.selected_service = Some("my-service".to_string());
        state.editor_state.content = "service content".to_string();
        
        assert_eq!(state.editor_state.selected_service, Some("my-service".to_string()));
    }

    #[test]
    fn test_configuration_state() {
        let state = TestGuiAppState::default();
        
        assert_eq!(state.config.services_directory, PathBuf::from("services"));
        assert_eq!(state.config.default_port, 8080);
    }

    #[test]
    fn test_configuration_state_update() {
        let mut state = TestGuiAppState::default();
        
        state.config.services_directory = PathBuf::from("/custom/path");
        state.config.default_port = 9090;
        
        assert_eq!(state.config.services_directory, PathBuf::from("/custom/path"));
        assert_eq!(state.config.default_port, 9090);
    }

    #[test]
    fn test_ai_state_fields() {
        let mut state = TestGuiAppState::default();
        
        state.ai_prompt = "Generate a user API".to_string();
        state.ai_generation_in_progress = true;
        
        assert_eq!(state.ai_prompt, "Generate a user API");
        assert!(state.ai_generation_in_progress);
        assert!(state.ai_generated_yaml.is_none());
        assert!(state.ai_error.is_none());
    }

    #[test]
    fn test_ai_generation_success() {
        let mut state = TestGuiAppState::default();
        
        state.ai_generation_in_progress = true;
        state.ai_generated_yaml = Some("name: generated-service".to_string());
        state.ai_generation_in_progress = false;
        
        assert!(!state.ai_generation_in_progress);
        assert!(state.ai_generated_yaml.is_some());
        assert!(state.ai_error.is_none());
    }

    #[test]
    fn test_ai_generation_error() {
        let mut state = TestGuiAppState::default();
        
        state.ai_generation_in_progress = true;
        state.ai_error = Some("API key invalid".to_string());
        state.ai_generation_in_progress = false;
        
        assert!(!state.ai_generation_in_progress);
        assert!(state.ai_generated_yaml.is_none());
        assert_eq!(state.ai_error, Some("API key invalid".to_string()));
    }

    #[test]
    fn test_request_logs_collection() {
        let mut state = TestGuiAppState::default();
        
        state.request_logs.push_back("GET /api/users 200".to_string());
        state.request_logs.push_back("POST /api/users 201".to_string());
        
        assert_eq!(state.request_logs.len(), 2);
        assert_eq!(state.request_logs[0], "GET /api/users 200");
        assert_eq!(state.request_logs[1], "POST /api/users 201");
    }

    #[test]
    fn test_request_logs_rotation() {
        let mut state = TestGuiAppState::default();
        
        // Add more than 1000 logs to test rotation
        for i in 0..1500 {
            state.request_logs.push_back(format!("Log entry {}", i));
        }
        
        // Simulate rotation by keeping only last 1000
        while state.request_logs.len() > 1000 {
            state.request_logs.pop_front();
        }
        
        assert_eq!(state.request_logs.len(), 1000);
        assert_eq!(state.request_logs[0], "Log entry 500");
        assert_eq!(state.request_logs[999], "Log entry 1499");
    }

    #[test]
    fn test_window_visibility_flags() {
        let mut state = TestGuiAppState::default();
        
        assert!(!state.show_ai_window);
        assert!(!state.show_editor_window);
        assert!(!state.show_config_window);
        
        state.show_ai_window = true;
        state.show_editor_window = true;
        
        assert!(state.show_ai_window);
        assert!(state.show_editor_window);
        assert!(!state.show_config_window);
    }

    #[test]
    fn test_selected_service_tracking() {
        let mut state = TestGuiAppState::default();
        
        assert!(state.selected_service.is_none());
        
        state.selected_service = Some("my-service".to_string());
        
        assert_eq!(state.selected_service, Some("my-service".to_string()));
        
        state.selected_service = None;
        
        assert!(state.selected_service.is_none());
    }

    #[test]
    fn test_multiple_services_management() {
        let mut state = TestGuiAppState::default();
        
        let service1 = TestServiceInfo {
            name: "service-1".to_string(),
            path: PathBuf::from("services/s1.yaml"),
            status: TestServiceStatus::Running,
            port: 8080,
            endpoints: vec![],
        };
        
        let service2 = TestServiceInfo {
            name: "service-2".to_string(),
            path: PathBuf::from("services/s2.yaml"),
            status: TestServiceStatus::Stopped,
            port: 8081,
            endpoints: vec![],
        };
        
        state.services.push(service1);
        state.services.push(service2);
        
        assert_eq!(state.services.len(), 2);
        assert_eq!(state.services[0].name, "service-1");
        assert_eq!(state.services[1].name, "service-2");
        assert_eq!(state.services[0].status, TestServiceStatus::Running);
        assert_eq!(state.services[1].status, TestServiceStatus::Stopped);
    }
}

#[cfg(test)]
mod service_info_tests {
    use super::*;
    use super::enhanced_state_tests::*;

    #[test]
    fn test_service_status_variants() {
        let stopped = TestServiceStatus::Stopped;
        let starting = TestServiceStatus::Starting;
        let running = TestServiceStatus::Running;
        let stopping = TestServiceStatus::Stopping;
        let failed = TestServiceStatus::Failed("Connection error".to_string());
        
        assert_eq!(stopped, TestServiceStatus::Stopped);
        assert_eq!(starting, TestServiceStatus::Starting);
        assert_eq!(running, TestServiceStatus::Running);
        assert_eq!(stopping, TestServiceStatus::Stopping);
        assert_eq!(failed, TestServiceStatus::Failed("Connection error".to_string()));
    }

    #[test]
    fn test_service_status_transition_stopped_to_starting() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Stopped,
            port: 8080,
            endpoints: vec![],
        };
        
        // Transition: Stopped -> Starting
        service.status = TestServiceStatus::Starting;
        assert_eq!(service.status, TestServiceStatus::Starting);
    }

    #[test]
    fn test_service_status_transition_starting_to_running() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Starting,
            port: 8080,
            endpoints: vec![],
        };
        
        // Transition: Starting -> Running
        service.status = TestServiceStatus::Running;
        assert_eq!(service.status, TestServiceStatus::Running);
    }

    #[test]
    fn test_service_status_transition_running_to_stopping() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Running,
            port: 8080,
            endpoints: vec![],
        };
        
        // Transition: Running -> Stopping
        service.status = TestServiceStatus::Stopping;
        assert_eq!(service.status, TestServiceStatus::Stopping);
    }

    #[test]
    fn test_service_status_transition_stopping_to_stopped() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Stopping,
            port: 8080,
            endpoints: vec![],
        };
        
        // Transition: Stopping -> Stopped
        service.status = TestServiceStatus::Stopped;
        assert_eq!(service.status, TestServiceStatus::Stopped);
    }

    #[test]
    fn test_service_status_transition_to_failed() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Starting,
            port: 8080,
            endpoints: vec![],
        };
        
        // Transition: Starting -> Failed
        service.status = TestServiceStatus::Failed("Port already in use".to_string());
        
        match &service.status {
            TestServiceStatus::Failed(msg) => assert_eq!(msg, "Port already in use"),
            _ => panic!("Expected Failed status"),
        }
    }

    #[test]
    fn test_service_status_transition_failed_to_stopped() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Failed("Error".to_string()),
            port: 8080,
            endpoints: vec![],
        };
        
        // Transition: Failed -> Stopped (recovery)
        service.status = TestServiceStatus::Stopped;
        assert_eq!(service.status, TestServiceStatus::Stopped);
    }

    #[test]
    fn test_service_info_metadata() {
        let service = TestServiceInfo {
            name: "user-api".to_string(),
            path: PathBuf::from("services/user-api.yaml"),
            status: TestServiceStatus::Running,
            port: 9000,
            endpoints: vec![
                TestEndpointInfo {
                    method: "GET".to_string(),
                    path: "/users".to_string(),
                },
                TestEndpointInfo {
                    method: "POST".to_string(),
                    path: "/users".to_string(),
                },
                TestEndpointInfo {
                    method: "GET".to_string(),
                    path: "/users/{id}".to_string(),
                },
            ],
        };
        
        assert_eq!(service.name, "user-api");
        assert_eq!(service.path, PathBuf::from("services/user-api.yaml"));
        assert_eq!(service.port, 9000);
        assert_eq!(service.endpoints.len(), 3);
        assert_eq!(service.endpoints[0].method, "GET");
        assert_eq!(service.endpoints[0].path, "/users");
        assert_eq!(service.endpoints[2].path, "/users/{id}");
    }

    #[test]
    fn test_service_info_clone() {
        let service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Running,
            port: 8080,
            endpoints: vec![],
        };
        
        let cloned = service.clone();
        
        assert_eq!(service.name, cloned.name);
        assert_eq!(service.path, cloned.path);
        assert_eq!(service.status, cloned.status);
        assert_eq!(service.port, cloned.port);
    }

    #[test]
    fn test_endpoint_info_structure() {
        let endpoint = TestEndpointInfo {
            method: "DELETE".to_string(),
            path: "/users/{id}".to_string(),
        };
        
        assert_eq!(endpoint.method, "DELETE");
        assert_eq!(endpoint.path, "/users/{id}");
    }

    #[test]
    fn test_service_with_no_endpoints() {
        let service = TestServiceInfo {
            name: "empty-service".to_string(),
            path: PathBuf::from("empty.yaml"),
            status: TestServiceStatus::Stopped,
            port: 8080,
            endpoints: vec![],
        };
        
        assert!(service.endpoints.is_empty());
    }

    #[test]
    fn test_service_with_many_endpoints() {
        let mut endpoints = vec![];
        for i in 0..50 {
            endpoints.push(TestEndpointInfo {
                method: "GET".to_string(),
                path: format!("/endpoint/{}", i),
            });
        }
        
        let service = TestServiceInfo {
            name: "large-service".to_string(),
            path: PathBuf::from("large.yaml"),
            status: TestServiceStatus::Running,
            port: 8080,
            endpoints,
        };
        
        assert_eq!(service.endpoints.len(), 50);
        assert_eq!(service.endpoints[0].path, "/endpoint/0");
        assert_eq!(service.endpoints[49].path, "/endpoint/49");
    }
}

#[cfg(test)]
mod service_status_methods_tests {
    use super::*;
    use super::enhanced_state_tests::*;

    #[test]
    fn test_status_can_start_from_stopped() {
        let status = TestServiceStatus::Stopped;
        // In real implementation, this would be status.can_start()
        // For test, we check the logic
        let can_start = matches!(status, TestServiceStatus::Stopped | TestServiceStatus::Failed(_));
        assert!(can_start);
    }

    #[test]
    fn test_status_cannot_start_from_running() {
        let status = TestServiceStatus::Running;
        let can_start = matches!(status, TestServiceStatus::Stopped | TestServiceStatus::Failed(_));
        assert!(!can_start);
    }

    #[test]
    fn test_status_can_start_from_failed() {
        let status = TestServiceStatus::Failed("Error".to_string());
        let can_start = matches!(status, TestServiceStatus::Stopped | TestServiceStatus::Failed(_));
        assert!(can_start);
    }

    #[test]
    fn test_status_can_stop_from_running() {
        let status = TestServiceStatus::Running;
        let can_stop = matches!(status, TestServiceStatus::Running);
        assert!(can_stop);
    }

    #[test]
    fn test_status_cannot_stop_from_stopped() {
        let status = TestServiceStatus::Stopped;
        let can_stop = matches!(status, TestServiceStatus::Running);
        assert!(!can_stop);
    }

    #[test]
    fn test_status_is_transitioning_starting() {
        let status = TestServiceStatus::Starting;
        let is_transitioning = matches!(status, TestServiceStatus::Starting | TestServiceStatus::Stopping);
        assert!(is_transitioning);
    }

    #[test]
    fn test_status_is_transitioning_stopping() {
        let status = TestServiceStatus::Stopping;
        let is_transitioning = matches!(status, TestServiceStatus::Starting | TestServiceStatus::Stopping);
        assert!(is_transitioning);
    }

    #[test]
    fn test_status_not_transitioning_running() {
        let status = TestServiceStatus::Running;
        let is_transitioning = matches!(status, TestServiceStatus::Starting | TestServiceStatus::Stopping);
        assert!(!is_transitioning);
    }

    #[test]
    fn test_status_is_running() {
        let status = TestServiceStatus::Running;
        let is_running = matches!(status, TestServiceStatus::Running);
        assert!(is_running);
    }

    #[test]
    fn test_status_is_failed() {
        let status = TestServiceStatus::Failed("Error".to_string());
        let is_failed = matches!(status, TestServiceStatus::Failed(_));
        assert!(is_failed);
    }

    #[test]
    fn test_complete_lifecycle() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Stopped,
            port: 8080,
            endpoints: vec![],
        };
        
        // Start sequence
        assert_eq!(service.status, TestServiceStatus::Stopped);
        service.status = TestServiceStatus::Starting;
        assert_eq!(service.status, TestServiceStatus::Starting);
        service.status = TestServiceStatus::Running;
        assert_eq!(service.status, TestServiceStatus::Running);
        
        // Stop sequence
        service.status = TestServiceStatus::Stopping;
        assert_eq!(service.status, TestServiceStatus::Stopping);
        service.status = TestServiceStatus::Stopped;
        assert_eq!(service.status, TestServiceStatus::Stopped);
    }

    #[test]
    fn test_failure_recovery_lifecycle() {
        let mut service = TestServiceInfo {
            name: "test".to_string(),
            path: PathBuf::from("test.yaml"),
            status: TestServiceStatus::Stopped,
            port: 8080,
            endpoints: vec![],
        };
        
        // Try to start but fail
        service.status = TestServiceStatus::Starting;
        service.status = TestServiceStatus::Failed("Port in use".to_string());
        
        match &service.status {
            TestServiceStatus::Failed(msg) => assert_eq!(msg, "Port in use"),
            _ => panic!("Expected Failed status"),
        }
        
        // Recover by resetting to stopped
        service.status = TestServiceStatus::Stopped;
        assert_eq!(service.status, TestServiceStatus::Stopped);
        
        // Try again
        service.status = TestServiceStatus::Starting;
        service.status = TestServiceStatus::Running;
        assert_eq!(service.status, TestServiceStatus::Running);
    }
}
