//! Event Handler for GUI Messages
//!
//! This module handles all GUI messages and coordinates between
//! the GUI state and the simulator manager.

#![cfg(feature = "gui")]

use super::messages::{GuiMessage, CapturedRequest, ExportFormat, CodeGenTarget};
use super::state::GuiAppState;
use super::models::{ServiceStatus, RequestLogEntry, ServiceInfo, EndpointInfo};
use apicentric::simulator::manager::ApiSimulatorManager;
use apicentric::{ApicentricError, ApicentricResult};
use std::sync::Arc;
use std::path::{Path, PathBuf};

/// Event handler for processing GUI messages
pub struct EventHandler {
    manager: Arc<ApiSimulatorManager>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(manager: Arc<ApiSimulatorManager>) -> Self {
        Self { manager }
    }

    /// Handle a GUI message and update state accordingly
    pub async fn handle_message(
        &self,
        message: GuiMessage,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        match message {
            // Service Management
            GuiMessage::StartService(name) => {
                self.handle_start_service(&name, state).await
            }
            GuiMessage::StopService(name) => {
                self.handle_stop_service(&name, state).await
            }
            GuiMessage::RefreshServices => {
                self.handle_refresh_services(state).await
            }
            GuiMessage::ServiceStatusChanged(name, status) => {
                self.handle_service_status_changed(&name, status, state).await
            }

            // AI Generation (existing functionality)
            GuiMessage::AiGenerate(_prompt) => {
                // This is handled in the main app loop with async spawn
                // Just mark as in progress here
                state.start_ai_generation();
                Ok(())
            }
            GuiMessage::AiGenerationComplete(result) => {
                match result {
                    Ok(yaml) => state.complete_ai_generation(yaml),
                    Err(e) => state.fail_ai_generation(e),
                }
                Ok(())
            }
            GuiMessage::AiApplyYaml(yaml) => {
                self.handle_ai_apply_yaml(&yaml, state).await
            }

            // Recording Mode
            GuiMessage::StartRecording(target_url) => {
                self.handle_start_recording(&target_url, state).await
            }
            GuiMessage::StopRecording => {
                self.handle_stop_recording(state).await
            }
            GuiMessage::CaptureRequest(request) => {
                self.handle_capture_request(request, state).await
            }
            GuiMessage::GenerateFromRecording => {
                self.handle_generate_from_recording(state).await
            }

            // Editor
            GuiMessage::LoadServiceInEditor(name) => {
                self.handle_load_service_in_editor(&name, state).await
            }
            GuiMessage::SaveEditorContent => {
                self.handle_save_editor_content(state).await
            }
            GuiMessage::EditorContentChanged(content) => {
                state.editor_state.content = content;
                state.mark_editor_dirty();
                Ok(())
            }

            // Logs
            GuiMessage::NewRequestLog(entry) => {
                self.handle_new_request_log(entry, state).await
            }
            GuiMessage::ClearLogs => {
                state.clear_logs();
                Ok(())
            }
            GuiMessage::FilterLogsBy(filter) => {
                state.set_log_filter(filter);
                Ok(())
            }

            // Import/Export
            GuiMessage::ImportFile(path) => {
                self.handle_import_file(&path, state).await
            }
            GuiMessage::ExportService(name, format) => {
                self.handle_export_service(&name, format, state).await
            }
            GuiMessage::BatchImport(paths) => {
                self.handle_batch_import(&paths, state).await
            }

            // Code Generation
            GuiMessage::GenerateCode(name, target) => {
                self.handle_generate_code(&name, target, state).await
            }
            GuiMessage::CopyToClipboard(content) => {
                self.handle_copy_to_clipboard(&content, state).await
            }
            GuiMessage::SaveGeneratedCode(path, content) => {
                self.handle_save_generated_code(&path, &content, state).await
            }

            // Configuration
            GuiMessage::UpdateConfig(config) => {
                state.config = config;
                Ok(())
            }
            GuiMessage::SaveConfig => {
                self.handle_save_config(state).await
            }
            GuiMessage::LoadConfig => {
                self.handle_load_config(state).await
            }
        }
    }

    // Service Management Handlers

    async fn handle_start_service(
        &self,
        name: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // Check if service exists and can be started
        {
            let service = state.find_service_mut(name);
            if service.is_none() {
                return Err(ApicentricError::runtime_error(
                    format!("Service not found: {}", name),
                    None::<String>,
                ));
            }
            
            // Update state to Starting
            let service = service.unwrap();
            service.start().map_err(|e| {
                ApicentricError::runtime_error(e, None::<String>)
            })?;
        }
        
        state.add_log(format!("Starting service: {}", name));
        
        // Actually start the service via manager
        let service_name = name.to_string();
        let manager = self.manager.clone();
        
        // Spawn async task to start service
        tokio::spawn(async move {
            match manager.start_service(&service_name).await {
                Ok(_) => {
                    log::info!("Service '{}' started successfully", service_name);
                }
                Err(e) => {
                    log::error!("Failed to start service '{}': {}", service_name, e);
                }
            }
        });
        
        // Mark as running (optimistic update)
        if let Some(service) = state.find_service_mut(name) {
            service.mark_running();
        }
        
        Ok(())
    }

    async fn handle_stop_service(
        &self,
        name: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // Check if service exists and can be stopped
        {
            let service = state.find_service_mut(name);
            if service.is_none() {
                return Err(ApicentricError::runtime_error(
                    format!("Service not found: {}", name),
                    None::<String>,
                ));
            }
            
            // Update state to Stopping
            let service = service.unwrap();
            service.stop().map_err(|e| {
                ApicentricError::runtime_error(e, None::<String>)
            })?;
        }
        
        state.add_log(format!("Stopping service: {}", name));
        
        // Actually stop the service via manager
        let service_name = name.to_string();
        let manager = self.manager.clone();
        
        // Spawn async task to stop service
        tokio::spawn(async move {
            match manager.stop_service(&service_name).await {
                Ok(_) => {
                    log::info!("Service '{}' stopped successfully", service_name);
                }
                Err(e) => {
                    log::error!("Failed to stop service '{}': {}", service_name, e);
                }
            }
        });
        
        // Mark as stopped (optimistic update)
        if let Some(service) = state.find_service_mut(name) {
            service.mark_stopped();
        }
        
        Ok(())
    }

    async fn handle_refresh_services(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        use apicentric::simulator::config::ConfigLoader;
        use std::collections::HashMap;
        
        // Scan services directory (clone to avoid borrow issues)
        let services_dir = state.config.services_directory.clone();
        
        if !services_dir.exists() {
            return Err(ApicentricError::runtime_error(
                format!("Services directory not found: {:?}", services_dir),
                Some("Create the services directory or update the configuration"),
            ));
        }
        
        // Load service definitions
        let config_loader = ConfigLoader::new(services_dir.clone());
        let service_definitions = match config_loader.load_all_services() {
            Ok(defs) => defs,
            Err(e) => {
                state.add_log(format!("Error loading services: {}", e));
                return Err(ApicentricError::runtime_error(
                    format!("Failed to load services: {}", e),
                    Some("Check service YAML files for errors"),
                ));
            }
        };
        
        // Keep track of existing running services
        let mut running_services: HashMap<String, ServiceStatus> = HashMap::new();
        for service in &state.services {
            if service.status.is_running() || service.status.is_transitioning() {
                running_services.insert(service.name.clone(), service.status.clone());
            }
        }
        
        // Clear and rebuild service list
        state.services.clear();
        
        // Add discovered services
        for def in service_definitions {
            let service_name = def.name.clone();
            let service_path = services_dir.join(format!("{}.yaml", service_name));
            let port = def.server.port.unwrap_or(state.config.default_port);
            
            let mut service_info = ServiceInfo::new(
                service_name.clone(),
                service_path,
                port,
            );
            
            // Parse endpoints from definition
            for endpoint in &def.endpoints {
                service_info.endpoints.push(EndpointInfo {
                    method: endpoint.method.clone(),
                    path: endpoint.path.clone(),
                });
            }
            
            // Restore status if service was running
            if let Some(status) = running_services.get(&service_name) {
                service_info.status = status.clone();
            }
            
            state.add_service(service_info);
        }
        
        state.add_log(format!("Discovered {} services", state.services.len()));
        Ok(())
    }

    async fn handle_service_status_changed(
        &self,
        name: &str,
        status: ServiceStatus,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        state.update_service_status(name, status);
        Ok(())
    }

    // AI Generation Handlers

    async fn handle_ai_apply_yaml(
        &self,
        _yaml: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Validate YAML and save to file
        // For now, just log
        state.add_log("Applied AI-generated YAML".to_string());
        Ok(())
    }

    // Recording Mode Handlers

    async fn handle_start_recording(
        &self,
        target_url: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Start proxy server and create recording session
        state.add_log(format!("Started recording: {}", target_url));
        Ok(())
    }

    async fn handle_stop_recording(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Stop proxy server and finalize recording
        state.recording_session = None;
        state.add_log("Stopped recording".to_string());
        Ok(())
    }

    async fn handle_capture_request(
        &self,
        _request: CapturedRequest,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Add request to recording session
        state.add_log("Captured request".to_string());
        Ok(())
    }

    async fn handle_generate_from_recording(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Convert captured requests to service definition
        state.add_log("Generated service from recording".to_string());
        Ok(())
    }

    // Editor Handlers

    async fn handle_load_service_in_editor(
        &self,
        name: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        if let Some(service) = state.find_service(name) {
            // TODO: Read actual file content
            let content = format!("# Service: {}\n# TODO: Load actual content from {:?}", name, service.path);
            state.load_service_in_editor(name.to_string(), content);
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                format!("Service not found: {}", name),
                None::<String>,
            ))
        }
    }

    async fn handle_save_editor_content(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        if let Some(service_name) = state.editor_state.selected_service.clone() {
            // TODO: Validate YAML and save to file
            state.mark_editor_clean();
            state.add_log(format!("Saved service: {}", service_name));
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                "No service selected in editor",
                None::<String>,
            ))
        }
    }

    // Log Handlers

    async fn handle_new_request_log(
        &self,
        entry: RequestLogEntry,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // Add to structured request logs
        state.add_request_log(entry.clone());
        
        // Also add to string logs for backward compatibility
        let log = format!(
            "{} {} {} - {}ms",
            entry.method,
            entry.path,
            entry.status_code,
            entry.duration_ms
        );
        state.add_log(log);
        Ok(())
    }

    // Import/Export Handlers

    async fn handle_import_file(
        &self,
        path: &Path,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Detect format and import
        state.add_log(format!("Imported file: {:?}", path));
        Ok(())
    }

    async fn handle_export_service(
        &self,
        name: &str,
        _format: ExportFormat,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Export service to specified format
        state.add_log(format!("Exported service: {}", name));
        Ok(())
    }

    async fn handle_batch_import(
        &self,
        paths: &[PathBuf],
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Import multiple files
        state.add_log(format!("Batch imported {} files", paths.len()));
        Ok(())
    }

    // Code Generation Handlers

    async fn handle_generate_code(
        &self,
        name: &str,
        _target: CodeGenTarget,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Generate code for service
        state.add_log(format!("Generated code for: {}", name));
        Ok(())
    }

    async fn handle_copy_to_clipboard(
        &self,
        _content: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Copy to system clipboard
        state.add_log("Copied to clipboard".to_string());
        Ok(())
    }

    async fn handle_save_generated_code(
        &self,
        path: &Path,
        _content: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Save generated code to file
        state.add_log(format!("Saved generated code to: {:?}", path));
        Ok(())
    }

    // Configuration Handlers

    async fn handle_save_config(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Serialize and save config to file
        state.add_log("Saved configuration".to_string());
        Ok(())
    }

    async fn handle_load_config(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        // TODO: Load config from file or use defaults
        state.add_log("Loaded configuration".to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> Arc<ApiSimulatorManager> {
        use apicentric::simulator::config::{SimulatorConfig, PortRange};
        let config = SimulatorConfig {
            enabled: true,
            services_dir: std::path::PathBuf::from("test_services"),
            port_range: PortRange { start: 9000, end: 9099 },
            db_path: std::path::PathBuf::from(":memory:"),
            admin_port: None,
            global_behavior: None,
        };
        Arc::new(ApiSimulatorManager::new(config))
    }

    #[tokio::test]
    async fn test_event_handler_creation() {
        let manager = create_test_manager();
        let _handler = EventHandler::new(manager);
        // Just verify it can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_handle_clear_logs() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);

        // Add some logs
        state.add_log("Log 1".to_string());
        state.add_log("Log 2".to_string());
        assert_eq!(state.logs.len(), 2);

        // Clear logs
        let result = handler.handle_message(GuiMessage::ClearLogs, &mut state).await;
        assert!(result.is_ok());
        assert_eq!(state.logs.len(), 0);
    }

    #[tokio::test]
    async fn test_handle_editor_content_changed() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);

        let new_content = "test content".to_string();
        let result = handler.handle_message(
            GuiMessage::EditorContentChanged(new_content.clone()),
            &mut state
        ).await;

        assert!(result.is_ok());
        assert_eq!(state.editor_state.content, new_content);
        assert!(state.editor_state.dirty);
    }

    #[tokio::test]
    async fn test_handle_update_config() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);

        let new_config = super::super::models::GuiConfig {
            services_directory: PathBuf::from("/new/path"),
            default_port: 9000,
        };

        let result = handler.handle_message(
            GuiMessage::UpdateConfig(new_config.clone()),
            &mut state
        ).await;

        assert!(result.is_ok());
        assert_eq!(state.config.services_directory, PathBuf::from("/new/path"));
        assert_eq!(state.config.default_port, 9000);
    }
    
    // Integration tests for service lifecycle
    
    #[tokio::test]
    async fn test_service_lifecycle_start_stop() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Add a service
        let service = super::super::models::ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );
        state.add_service(service);
        
        // Start the service
        let result = handler.handle_message(
            GuiMessage::StartService("test-service".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_ok());
        
        // Verify service is running
        let service = state.find_service("test-service").unwrap();
        assert!(service.status.is_running());
        
        // Stop the service
        let result = handler.handle_message(
            GuiMessage::StopService("test-service".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_ok());
        
        // Verify service is stopped
        let service = state.find_service("test-service").unwrap();
        assert_eq!(service.status, super::super::models::ServiceStatus::Stopped);
    }
    
    #[tokio::test]
    async fn test_service_lifecycle_multiple_services() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Add multiple services
        let service1 = super::super::models::ServiceInfo::new(
            "service-1".to_string(),
            PathBuf::from("services/s1.yaml"),
            8080,
        );
        let service2 = super::super::models::ServiceInfo::new(
            "service-2".to_string(),
            PathBuf::from("services/s2.yaml"),
            8081,
        );
        
        state.add_service(service1);
        state.add_service(service2);
        
        // Start both services
        handler.handle_message(
            GuiMessage::StartService("service-1".to_string()),
            &mut state
        ).await.unwrap();
        
        handler.handle_message(
            GuiMessage::StartService("service-2".to_string()),
            &mut state
        ).await.unwrap();
        
        // Verify both are running
        assert!(state.find_service("service-1").unwrap().status.is_running());
        assert!(state.find_service("service-2").unwrap().status.is_running());
        
        // Stop first service
        handler.handle_message(
            GuiMessage::StopService("service-1".to_string()),
            &mut state
        ).await.unwrap();
        
        // Verify first is stopped, second still running
        assert_eq!(state.find_service("service-1").unwrap().status, super::super::models::ServiceStatus::Stopped);
        assert!(state.find_service("service-2").unwrap().status.is_running());
    }
    
    #[tokio::test]
    async fn test_service_lifecycle_error_nonexistent() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Try to start nonexistent service
        let result = handler.handle_message(
            GuiMessage::StartService("nonexistent".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_err());
        
        // Try to stop nonexistent service
        let result = handler.handle_message(
            GuiMessage::StopService("nonexistent".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_service_lifecycle_error_already_running() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Add and start a service
        let service = super::super::models::ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );
        state.add_service(service);
        
        handler.handle_message(
            GuiMessage::StartService("test-service".to_string()),
            &mut state
        ).await.unwrap();
        
        // Try to start again
        let result = handler.handle_message(
            GuiMessage::StartService("test-service".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_service_lifecycle_error_already_stopped() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Add a stopped service
        let service = super::super::models::ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );
        state.add_service(service);
        
        // Try to stop already stopped service
        let result = handler.handle_message(
            GuiMessage::StopService("test-service".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_service_lifecycle_status_changed() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Add a service
        let service = super::super::models::ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );
        state.add_service(service);
        
        // Simulate status changes
        handler.handle_message(
            GuiMessage::ServiceStatusChanged(
                "test-service".to_string(),
                super::super::models::ServiceStatus::Starting
            ),
            &mut state
        ).await.unwrap();

        assert_eq!(
            state.find_service("test-service").unwrap().status,
            super::super::models::ServiceStatus::Starting
        );

        handler.handle_message(
            GuiMessage::ServiceStatusChanged(
                "test-service".to_string(),
                super::super::models::ServiceStatus::Running
            ),
            &mut state
        ).await.unwrap();
        
        assert!(state.find_service("test-service").unwrap().status.is_running());
    }
    
    #[tokio::test]
    async fn test_service_lifecycle_failure_recovery() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        
        // Add a service
        let service = super::super::models::ServiceInfo::new(
            "test-service".to_string(),
            PathBuf::from("services/test.yaml"),
            8080,
        );
        state.add_service(service);
        
        // Simulate failure
        handler.handle_message(
            GuiMessage::ServiceStatusChanged(
                "test-service".to_string(),
                super::super::models::ServiceStatus::Failed("Port in use".to_string())
            ),
            &mut state
        ).await.unwrap();
        
        assert!(state.find_service("test-service").unwrap().status.is_failed());
        
        // Should be able to restart from failed state
        let result = handler.handle_message(
            GuiMessage::StartService("test-service".to_string()),
            &mut state
        ).await;
        
        assert!(result.is_ok());
        assert!(state.find_service("test-service").unwrap().status.is_running());
    }
}
