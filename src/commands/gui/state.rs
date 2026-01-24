//! GUI Application State Logic
//!
//! This module contains the state management logic for the GUI application.
//! Data models are defined in the `models` module.

#![allow(dead_code)]

use super::messages::GuiSystemEvent;
use super::models::*;
use std::collections::VecDeque;
use std::sync::mpsc;
use tokio::sync::broadcast;

/// Main GUI application state
pub struct GuiAppState {
    // Service management
    pub services: Vec<ServiceInfo>,
    pub selected_service: Option<String>,
    pub refreshing_services: bool,

    // AI generation (existing fields preserved and enhanced)
    pub ai_prompt: String,
    pub ai_generated_yaml: Option<String>,
    pub ai_generation_in_progress: bool,
    pub ai_error: Option<String>,
    pub ai_validation_errors: Vec<String>,
    pub ai_config_missing: bool,

    // Request logs (enhanced from Vec<String> to VecDeque)
    pub logs: Vec<String>, // Keep for backward compatibility
    pub request_logs: VecDeque<RequestLogEntry>,
    pub log_filter: LogFilter,
    // pub log_receiver: broadcast::Receiver<apicentric::simulator::log::RequestLogEntry>,

    // System events channel
    pub system_event_rx: mpsc::Receiver<GuiSystemEvent>,
    pub system_event_tx: mpsc::Sender<GuiSystemEvent>,

    // Recording mode
    pub recording_session: Option<RecordingSession>,

    // Editor state
    pub editor_state: EditorState,

    // Code generation state
    pub codegen_state: GuiCodegenState,

    // Configuration
    pub config: GuiConfig,

    // UI state
    pub show_ai_window: bool,
    pub show_editor_window: bool,
    pub show_config_window: bool,

    // Simulator status
    pub is_simulator_running: bool,
}

impl GuiAppState {
    /// Helper function to rotate logs to keep only the last max_entries
    fn rotate_logs<T>(logs: &mut VecDeque<T>, max_entries: usize) {
        while logs.len() > max_entries {
            logs.pop_front();
        }
    }

    /// Helper function to rotate Vec logs by removing from the front
    fn rotate_vec_logs<T>(logs: &mut Vec<T>, max_entries: usize) {
        while logs.len() > max_entries {
            logs.remove(0);
        }
    }

    /// Helper to reset AI generation state to initial values
    fn reset_ai_generation_state(&mut self) {
        self.ai_generation_in_progress = false;
        self.ai_error = None;
        self.ai_generated_yaml = None;
        self.ai_validation_errors.clear();
        self.ai_config_missing = false;
    }

    /// Helper to reset editor loading state
    fn reset_editor_loading_state(&mut self) {
        self.editor_state.loading = false;
        self.editor_state.saving = false;
    }

    /// Helper to log operation results
    fn log_operation_result(&mut self, operation: &str, success: bool, details: Option<&str>) {
        let message = match (success, details) {
            (true, Some(d)) => format!("{}: {}", operation, d),
            (true, None) => operation.to_string(),
            (false, Some(d)) => format!("Failed to {}: {}", operation, d),
            (false, None) => format!("Failed to {}", operation),
        };
        self.add_log(message);
    }

    pub fn new(
        _log_receiver: broadcast::Receiver<apicentric::simulator::log::RequestLogEntry>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            services: Vec::new(),
            selected_service: None,
            refreshing_services: false,
            ai_prompt: String::new(),
            ai_generated_yaml: None,
            ai_generation_in_progress: false,
            ai_error: None,
            ai_validation_errors: Vec::new(),
            ai_config_missing: false,
            logs: Vec::new(), // Keep for backward compatibility
            request_logs: VecDeque::new(),
            log_filter: LogFilter::default(),
            // log_receiver,
            system_event_rx: rx,
            system_event_tx: tx,
            recording_session: None,
            editor_state: EditorState::default(),
            codegen_state: GuiCodegenState::default(),
            config: GuiConfig::default(),
            show_ai_window: false,
            show_editor_window: false,
            show_config_window: false,
            is_simulator_running: false,
        }
    }

    /// Add a service to the state
    pub fn add_service(&mut self, service: ServiceInfo) {
        self.services.push(service);
    }

    /// Remove a service by name
    pub fn remove_service(&mut self, name: &str) -> Option<ServiceInfo> {
        if let Some(pos) = self.services.iter().position(|s| s.name == name) {
            Some(self.services.remove(pos))
        } else {
            None
        }
    }

    /// Find a service by name
    pub fn find_service(&self, name: &str) -> Option<&ServiceInfo> {
        self.services.iter().find(|s| s.name == name)
    }

    /// Find a mutable service by name
    pub fn find_service_mut(&mut self, name: &str) -> Option<&mut ServiceInfo> {
        self.services.iter_mut().find(|s| s.name == name)
    }

    /// Update service status
    pub fn update_service_status(&mut self, name: &str, status: ServiceStatus) {
        if let Some(service) = self.find_service_mut(name) {
            service.status = status;
        }
    }

    /// Add a log entry with rotation (string version for backward compatibility)
    pub fn add_log(&mut self, log: String) {
        self.logs.push(log); // Keep for backward compatibility

        // Also rotate old logs vector
        Self::rotate_vec_logs(&mut self.logs, 1000);
    }

    /// Add a request log entry with rotation
    pub fn add_request_log(&mut self, entry: RequestLogEntry) {
        self.request_logs.push_back(entry);

        // Rotate logs to keep only last 1000 entries
        Self::rotate_logs(&mut self.request_logs, 1000);
    }

    /// Get filtered request logs
    pub fn filtered_request_logs(&self) -> Vec<RequestLogEntry> {
        self.request_logs
            .iter()
            .filter(|entry| self.log_filter.matches(entry))
            .cloned()
            .collect()
    }

    /// Set the log filter
    pub fn set_log_filter(&mut self, filter: LogFilter) {
        self.log_filter = filter;
    }

    /// Clear all logs
    pub fn clear_logs(&mut self) {
        self.request_logs.clear();
        self.logs.clear();
    }

    /// Get the number of request logs
    pub fn request_log_count(&self) -> usize {
        self.request_logs.len()
    }

    /// Start AI generation
    pub fn start_ai_generation(&mut self) {
        self.reset_ai_generation_state();
        self.ai_generation_in_progress = true;
    }

    /// Complete AI generation with success
    pub fn complete_ai_generation(&mut self, yaml: String) {
        self.reset_ai_generation_state();
        self.ai_generated_yaml = Some(yaml);
    }

    /// Complete AI generation with error
    pub fn fail_ai_generation(&mut self, error: String) {
        self.reset_ai_generation_state();

        // Check if error is due to missing configuration
        if error.contains("AI provider not configured") || error.contains("API key missing") {
            self.ai_config_missing = true;
        }
        self.ai_error = Some(error);
    }

    /// Set validation errors for generated YAML
    pub fn _set_validation_errors(&mut self, errors: Vec<String>) {
        self.ai_validation_errors = errors;
    }

    /// Clear validation errors
    pub fn _clear_validation_errors(&mut self) {
        self.ai_validation_errors.clear();
    }

    /// Check if AI configuration is present
    pub fn _check_ai_config(&mut self) {
        // This would be called on startup to check if AI is configured
        // For now, we'll set it based on whether we can load config
        match apicentric::config::load_config(std::path::Path::new("apicentric.json")) {
            Ok(cfg) => {
                self.ai_config_missing = cfg.ai.is_none();
            }
            Err(_) => {
                self.ai_config_missing = true;
            }
        }
    }

    /// Load service content into editor
    pub fn load_service_in_editor(&mut self, name: String, content: String) {
        self.editor_state.selected_service = Some(name);
        self.editor_state.content = content;
        self.editor_state.dirty = false;
        self.show_editor_window = true;
    }

    /// Mark editor as dirty
    pub fn mark_editor_dirty(&mut self) {
        self.editor_state.dirty = true;
    }

    /// Mark editor as clean (after save)
    pub fn mark_editor_clean(&mut self) {
        self.editor_state.dirty = false;
    }

    /// Start loading a service in editor
    pub fn start_loading_service(&mut self, service_name: String) {
        self.editor_state.selected_service = Some(service_name);
        self.reset_editor_loading_state();
        self.editor_state.loading = true;
        self.show_editor_window = true;
    }

    /// Complete loading a service in editor
    pub fn complete_loading_service(&mut self, content: String) {
        self.editor_state.content = content;
        self.reset_editor_loading_state();
        self.editor_state.dirty = false;
    }

    /// Fail loading a service in editor
    pub fn fail_loading_service(&mut self, error: String) {
        self.reset_editor_loading_state();
        self.log_operation_result("load service", false, Some(&error));
    }

    /// Start saving editor content
    pub fn start_saving_editor(&mut self) {
        self.reset_editor_loading_state();
        self.editor_state.saving = true;
    }

    /// Complete saving editor content
    pub fn complete_saving_editor(&mut self) {
        self.reset_editor_loading_state();
        self.editor_state.dirty = false;
    }

    /// Fail saving editor content
    pub fn fail_saving_editor(&mut self, error: String) {
        self.reset_editor_loading_state();
        self.log_operation_result("save service", false, Some(&error));
    }

    /// Start refreshing services
    pub fn start_refreshing_services(&mut self) {
        self.refreshing_services = true;
    }

    /// Complete refreshing services
    pub fn complete_refreshing_services(&mut self, services: Vec<ServiceInfo>) {
        self.services = services;
        self.refreshing_services = false;
        self.log_operation_result(
            "refresh services",
            true,
            Some(&format!("{}", self.services.len())),
        );
    }

    /// Fail refreshing services
    pub fn fail_refreshing_services(&mut self, error: String) {
        self.refreshing_services = false;
        self.log_operation_result("refresh services", false, Some(&error));
    }
}

#[cfg(test)]
mod service_management_tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to create a test state
    fn create_test_state() -> GuiAppState {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        GuiAppState::new(rx)
    }

    // Service Discovery Tests

    #[test]
    fn test_add_service() {
        let mut state = create_test_state();

        let service = ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );

        state.add_service(service);

        assert_eq!(state.services.len(), 1);
        assert_eq!(state.services[0].name, "test-service");
        assert_eq!(state.services[0].port, 8080);
        assert_eq!(state.services[0].status, ServiceStatus::Stopped);
    }

    #[test]
    fn test_add_multiple_services() {
        let mut state = create_test_state();

        let service1 = ServiceInfo::new(
            "service-1".to_string(),
            PathBuf::from("services/s1.yaml"),
            8080,
        );

        let service2 = ServiceInfo::new(
            "service-2".to_string(),
            PathBuf::from("services/s2.yaml"),
            8081,
        );

        state.add_service(service1);
        state.add_service(service2);

        assert_eq!(state.services.len(), 2);
        assert_eq!(state.services[0].name, "service-1");
        assert_eq!(state.services[1].name, "service-2");
    }

    #[test]
    fn test_find_service() {
        let mut state = create_test_state();

        let service = ServiceInfo::new(
            "my-service".to_string(),
            PathBuf::from("services/my.yaml"),
            8080,
        );
        state.add_service(service);

        let found = state.find_service("my-service");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "my-service");

        let not_found = state.find_service("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_service_mut() {
        let mut state = create_test_state();

        let service = ServiceInfo::new(
            "my-service".to_string(),
            PathBuf::from("services/my.yaml"),
            8080,
        );
        state.add_service(service);

        if let Some(svc) = state.find_service_mut("my-service") {
            svc.port = 9000;
        }

        assert_eq!(state.services[0].port, 9000);
    }

    #[test]
    fn test_remove_service() {
        let mut state = create_test_state();

        let service = ServiceInfo::new(
            "temp-service".to_string(),
            PathBuf::from("services/temp.yaml"),
            8080,
        );
        state.add_service(service);

        assert_eq!(state.services.len(), 1);

        let removed = state.remove_service("temp-service");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "temp-service");
        assert_eq!(state.services.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_service() {
        let mut state = create_test_state();

        let removed = state.remove_service("nonexistent");
        assert!(removed.is_none());
    }

    // Service Start Tests

    #[test]
    fn test_service_start_from_stopped() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        assert_eq!(service.status, ServiceStatus::Stopped);
        assert!(service.can_start());

        let result = service.start();
        assert!(result.is_ok());
        assert_eq!(service.status, ServiceStatus::Starting);
    }

    #[test]
    fn test_service_start_already_running() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        service.start().unwrap();
        service.mark_running();

        let result = service.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_start_from_failed() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        service.mark_failed("Error".to_string());
        assert!(service.can_start());

        let result = service.start();
        assert!(result.is_ok());
        assert_eq!(service.status, ServiceStatus::Starting);
    }

    #[test]
    fn test_service_mark_running() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        service.start().unwrap();
        service.mark_running();

        assert_eq!(service.status, ServiceStatus::Running);
        assert!(service.status.is_running());
    }

    // Service Stop Tests

    #[test]
    fn test_service_stop_from_running() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        service.start().unwrap();
        service.mark_running();
        assert!(service.can_stop());

        let result = service.stop();
        assert!(result.is_ok());
        assert_eq!(service.status, ServiceStatus::Stopping);
    }

    #[test]
    fn test_service_stop_already_stopped() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        assert!(!service.can_stop());

        let result = service.stop();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_mark_stopped() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        service.start().unwrap();
        service.mark_running();
        service.stop().unwrap();
        service.mark_stopped();

        assert_eq!(service.status, ServiceStatus::Stopped);
    }

    // Service Status Update Tests

    #[test]
    fn test_update_service_status() {
        let mut state = create_test_state();

        let service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);
        state.add_service(service);

        state.update_service_status("test", ServiceStatus::Running);

        let svc = state.find_service("test").unwrap();
        assert_eq!(svc.status, ServiceStatus::Running);
    }

    #[test]
    fn test_update_service_status_lifecycle() {
        let mut state = create_test_state();

        let service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);
        state.add_service(service);

        // Simulate full lifecycle
        state.update_service_status("test", ServiceStatus::Starting);
        assert_eq!(
            state.find_service("test").unwrap().status,
            ServiceStatus::Starting
        );

        state.update_service_status("test", ServiceStatus::Running);
        assert_eq!(
            state.find_service("test").unwrap().status,
            ServiceStatus::Running
        );

        state.update_service_status("test", ServiceStatus::Stopping);
        assert_eq!(
            state.find_service("test").unwrap().status,
            ServiceStatus::Stopping
        );

        state.update_service_status("test", ServiceStatus::Stopped);
        assert_eq!(
            state.find_service("test").unwrap().status,
            ServiceStatus::Stopped
        );
    }

    #[test]
    fn test_update_service_status_to_failed() {
        let mut state = create_test_state();

        let service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);
        state.add_service(service);

        state.update_service_status("test", ServiceStatus::Failed("Port in use".to_string()));

        let svc = state.find_service("test").unwrap();
        assert!(svc.status.is_failed());
        assert_eq!(svc.status.error_message(), Some("Port in use"));
    }

    // Service Status Tests

    #[test]
    fn test_service_status_can_start() {
        assert!(ServiceStatus::Stopped.can_start());
        assert!(!ServiceStatus::Starting.can_start());
        assert!(!ServiceStatus::Running.can_start());
        assert!(!ServiceStatus::Stopping.can_start());
        assert!(ServiceStatus::Failed("Error".to_string()).can_start());
    }

    #[test]
    fn test_service_status_can_stop() {
        assert!(!ServiceStatus::Stopped.can_stop());
        assert!(!ServiceStatus::Starting.can_stop());
        assert!(ServiceStatus::Running.can_stop());
        assert!(!ServiceStatus::Stopping.can_stop());
        assert!(!ServiceStatus::Failed("Error".to_string()).can_stop());
    }

    #[test]
    fn test_service_status_is_transitioning() {
        assert!(!ServiceStatus::Stopped.is_transitioning());
        assert!(ServiceStatus::Starting.is_transitioning());
        assert!(!ServiceStatus::Running.is_transitioning());
        assert!(ServiceStatus::Stopping.is_transitioning());
        assert!(!ServiceStatus::Failed("Error".to_string()).is_transitioning());
    }

    #[test]
    fn test_service_status_is_running() {
        assert!(!ServiceStatus::Stopped.is_running());
        assert!(!ServiceStatus::Starting.is_running());
        assert!(ServiceStatus::Running.is_running());
        assert!(!ServiceStatus::Stopping.is_running());
        assert!(!ServiceStatus::Failed("Error".to_string()).is_running());
    }

    #[test]
    fn test_service_status_is_failed() {
        assert!(!ServiceStatus::Stopped.is_failed());
        assert!(!ServiceStatus::Starting.is_failed());
        assert!(!ServiceStatus::Running.is_failed());
        assert!(!ServiceStatus::Stopping.is_failed());
        assert!(ServiceStatus::Failed("Error".to_string()).is_failed());
    }

    #[test]
    fn test_service_status_display_string() {
        assert_eq!(ServiceStatus::Stopped.display_string(), "Stopped");
        assert_eq!(ServiceStatus::Starting.display_string(), "Starting...");
        assert_eq!(ServiceStatus::Running.display_string(), "Running");
        assert_eq!(ServiceStatus::Stopping.display_string(), "Stopping...");
        assert_eq!(
            ServiceStatus::Failed("Error".to_string()).display_string(),
            "Failed"
        );
    }

    #[test]
    fn test_service_status_error_message() {
        assert_eq!(ServiceStatus::Stopped.error_message(), None);
        assert_eq!(ServiceStatus::Running.error_message(), None);
        assert_eq!(
            ServiceStatus::Failed("Test error".to_string()).error_message(),
            Some("Test error")
        );
    }

    // Service Info Tests

    #[test]
    fn test_service_info_creation() {
        let service = ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );

        assert_eq!(service.name, "test-service");
        assert_eq!(service.path, PathBuf::from("services/test.yaml"));
        assert_eq!(service.port, 8080);
        assert_eq!(service.status, ServiceStatus::Stopped);
        assert!(service.endpoints.is_empty());
    }

    #[test]
    fn test_service_info_with_endpoints() {
        let mut service = ServiceInfo::new("api".to_string(), PathBuf::from("api.yaml"), 9000);

        service.endpoints.push(EndpointInfo {
            method: "GET".to_string(),
            path: "/users".to_string(),
        });
        service.endpoints.push(EndpointInfo {
            method: "POST".to_string(),
            path: "/users".to_string(),
        });

        assert_eq!(service.endpoints.len(), 2);
        assert_eq!(service.endpoints[0].method, "GET");
        assert_eq!(service.endpoints[1].method, "POST");
    }

    #[test]
    fn test_service_failure_recovery() {
        let mut service = ServiceInfo::new("test".to_string(), PathBuf::from("test.yaml"), 8080);

        // Start and fail
        service.start().unwrap();
        service.mark_failed("Connection error".to_string());
        assert!(service.status.is_failed());

        // Should be able to restart
        let result = service.start();
        assert!(result.is_ok());
        assert_eq!(service.status, ServiceStatus::Starting);

        // Complete startup
        service.mark_running();
        assert!(service.status.is_running());
    }
}

#[cfg(test)]
mod log_integration_tests {
    use super::*;
    use std::time::SystemTime;

    // Helper to create a test state
    fn create_test_state() -> GuiAppState {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        GuiAppState::new(rx)
    }

    #[test]
    fn test_add_request_log() {
        let mut state = create_test_state();

        let entry = RequestLogEntry::new(
            "test-service".to_string(),
            "GET".to_string(),
            "/api/users".to_string(),
            200,
            45,
        );

        state.add_request_log(entry.clone());

        assert_eq!(state.request_log_count(), 1);
        assert_eq!(state.request_logs[0].service_name, "test-service");
        assert_eq!(state.request_logs[0].method, "GET");
        assert_eq!(state.request_logs[0].path, "/api/users");
        assert_eq!(state.request_logs[0].status_code, 200);
    }

    #[test]
    fn test_request_log_rotation() {
        let mut state = create_test_state();

        // Add 1500 logs
        for i in 0..1500 {
            let entry = RequestLogEntry::new(
                "service".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            state.add_request_log(entry);
        }

        // Should only keep last 1000
        assert_eq!(state.request_log_count(), 1000);
        assert_eq!(state.request_logs[0].path, "/path500");
        assert_eq!(state.request_logs[999].path, "/path1499");
    }

    #[test]
    fn test_filtered_request_logs() {
        let mut state = create_test_state();

        // Add logs for different services
        for i in 0..10 {
            let entry = RequestLogEntry::new(
                "service-a".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            state.add_request_log(entry);
        }

        for i in 0..5 {
            let entry = RequestLogEntry::new(
                "service-b".to_string(),
                "POST".to_string(),
                format!("/data{}", i),
                201,
                20,
            );
            state.add_request_log(entry);
        }

        // Filter by service-a
        state.set_log_filter(LogFilter::Service("service-a".to_string()));
        let filtered = state.filtered_request_logs();

        assert_eq!(filtered.len(), 10);
        assert!(filtered.iter().all(|e| e.service_name == "service-a"));
    }

    #[test]
    fn test_filter_by_status_code() {
        let mut state = create_test_state();

        // Add successful requests
        for i in 0..7 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/ok{}", i),
                200,
                10,
            );
            state.add_request_log(entry);
        }

        // Add error requests
        for i in 0..3 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/error{}", i),
                404,
                5,
            );
            state.add_request_log(entry);
        }

        // Filter by 404
        state.set_log_filter(LogFilter::StatusCode(404));
        let filtered = state.filtered_request_logs();

        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|e| e.status_code == 404));
    }

    #[test]
    fn test_filter_by_method() {
        let mut state = create_test_state();

        // Add GET requests
        for i in 0..6 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/get{}", i),
                200,
                10,
            );
            state.add_request_log(entry);
        }

        // Add POST requests
        for i in 0..4 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "POST".to_string(),
                format!("/post{}", i),
                201,
                20,
            );
            state.add_request_log(entry);
        }

        // Filter by POST
        state.set_log_filter(LogFilter::Method("POST".to_string()));
        let filtered = state.filtered_request_logs();

        assert_eq!(filtered.len(), 4);
        assert!(filtered.iter().all(|e| e.method == "POST"));
    }

    #[test]
    fn test_clear_request_logs() {
        let mut state = create_test_state();

        // Add some logs
        for i in 0..10 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            state.add_request_log(entry);
        }

        assert_eq!(state.request_log_count(), 10);

        state.clear_logs();

        assert_eq!(state.request_log_count(), 0);
    }

    #[test]
    fn test_log_filter_all() {
        let mut state = create_test_state();

        // Add various logs
        state.add_request_log(RequestLogEntry::new(
            "service-a".to_string(),
            "GET".to_string(),
            "/path1".to_string(),
            200,
            10,
        ));

        state.add_request_log(RequestLogEntry::new(
            "service-b".to_string(),
            "POST".to_string(),
            "/path2".to_string(),
            404,
            20,
        ));

        // Filter All should return everything
        state.set_log_filter(LogFilter::All);
        let filtered = state.filtered_request_logs();

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_from_simulator_log_conversion() {
        let sim_log = apicentric::simulator::log::RequestLogEntry::new(
            "my-service".to_string(),
            Some(1),
            "POST".to_string(),
            "/api/data".to_string(),
            201,
        );

        let gui_log = RequestLogEntry::from_simulator_log(&sim_log);

        assert_eq!(gui_log.service_name, "my-service");
        assert_eq!(gui_log.method, "POST");
        assert_eq!(gui_log.path, "/api/data");
        assert_eq!(gui_log.status_code, 201);
        assert_eq!(gui_log.duration_ms, 0);
    }

    #[tokio::test]
    async fn test_log_receiver_integration() {
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        let mut state = GuiAppState::new(rx);

        // Simulate receiving logs from simulator
        for i in 0..5 {
            let sim_log = apicentric::simulator::log::RequestLogEntry::new(
                "test-service".to_string(),
                Some(0),
                "GET".to_string(),
                format!("/endpoint{}", i),
                200,
            );

            tx.send(sim_log.clone()).unwrap();

            // In real app, this would be done in the update loop
            if let Ok(log) = state.log_receiver.try_recv() {
                let gui_log = RequestLogEntry::from_simulator_log(&log);
                state.add_request_log(gui_log);
            }
        }

        assert_eq!(state.request_log_count(), 5);
    }

    #[test]
    fn test_performance_with_many_request_logs() {
        let mut state = create_test_state();

        let start = SystemTime::now();

        // Add 1000 logs
        for i in 0..1000 {
            let entry = RequestLogEntry::new(
                format!("service-{}", i % 10),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            state.add_request_log(entry);
        }

        let elapsed = start.elapsed().unwrap();

        // Should complete quickly (under 50ms)
        assert!(elapsed < std::time::Duration::from_millis(50));
        assert_eq!(state.request_log_count(), 1000);
    }
}
