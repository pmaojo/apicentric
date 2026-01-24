//! Event Handler for GUI Messages
//!
//! This module handles all GUI messages and coordinates between
//! the GUI state and the simulator manager.

#![cfg(feature = "gui")]

use super::messages::{CapturedRequest, CodeGenTarget, ExportFormat, GuiMessage};
use super::models::{EndpointInfo, RequestLogEntry, ServiceInfo, ServiceStatus};
use super::state::GuiAppState;
use apicentric::simulator::config::{
    EndpointDefinition, ResponseDefinition, ServerConfig, ServiceDefinition,
};
use apicentric::simulator::manager::ApiSimulatorManager;
use apicentric::{ApicentricError, ApicentricResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
            GuiMessage::StartService(name) => self.handle_start_service(&name, state).await,
            GuiMessage::StopService(name) => self.handle_stop_service(&name, state).await,
            GuiMessage::RefreshServices => self.handle_refresh_services(state).await,
            GuiMessage::ServiceStatusChanged(name, status) => {
                self.handle_service_status_changed(&name, status, state)
                    .await
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
            GuiMessage::AiApplyYaml(yaml) => self.handle_ai_apply_yaml(&yaml, state).await,

            // Recording Mode
            GuiMessage::StartRecording(target_url) => {
                self.handle_start_recording(&target_url, state).await
            }
            GuiMessage::StopRecording => self.handle_stop_recording(state).await,
            GuiMessage::CaptureRequest(request) => {
                self.handle_capture_request(request, state).await
            }
            GuiMessage::GenerateFromRecording => self.handle_generate_from_recording(state).await,

            // Editor
            GuiMessage::LoadServiceInEditor(name) => {
                self.handle_load_service_in_editor(&name, state).await
            }
            GuiMessage::SaveEditorContent => self.handle_save_editor_content(state).await,
            GuiMessage::EditorContentChanged(content) => {
                state.editor_state.content = content;
                state.mark_editor_dirty();
                Ok(())
            }

            // Logs
            GuiMessage::NewRequestLog(entry) => self.handle_new_request_log(entry, state).await,
            GuiMessage::ClearLogs => {
                state.clear_logs();
                Ok(())
            }
            GuiMessage::FilterLogsBy(filter) => {
                state.set_log_filter(filter);
                Ok(())
            }

            // Import/Export
            GuiMessage::ImportFile(path) => self.handle_import_file(&path, state).await,
            GuiMessage::ExportService(name, format) => {
                self.handle_export_service(&name, format, state).await
            }
            GuiMessage::BatchImport(paths) => self.handle_batch_import(&paths, state).await,

            // Code Generation
            GuiMessage::GenerateCode(name, target) => {
                self.handle_generate_code(&name, target, state).await
            }
            GuiMessage::CopyToClipboard(content) => {
                self.handle_copy_to_clipboard(&content, state).await
            }
            GuiMessage::SaveGeneratedCode(path, content) => {
                self.handle_save_generated_code(&path, &content, state)
                    .await
            }

            // Configuration
            GuiMessage::UpdateConfig(config) => {
                state.config = config;
                Ok(())
            }
            GuiMessage::SaveConfig => self.handle_save_config(state).await,
            GuiMessage::LoadConfig => self.handle_load_config(state).await,
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
            service
                .start()
                .map_err(|e| ApicentricError::runtime_error(e, None::<String>))?;
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
            service
                .stop()
                .map_err(|e| ApicentricError::runtime_error(e, None::<String>))?;
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

    async fn handle_refresh_services(&self, state: &mut GuiAppState) -> ApicentricResult<()> {
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
            let port = def
                .server
                .as_ref()
                .and_then(|s| s.port)
                .unwrap_or(state.config.default_port);

            let mut service_info = ServiceInfo::new(service_name.clone(), service_path, port);

            // Parse endpoints from definition
            if let Some(endpoints) = &def.endpoints {
                for endpoint in endpoints {
                    service_info.endpoints.push(EndpointInfo {
                        method: endpoint.method.clone(),
                        path: endpoint.path.clone(),
                    });
                }
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
        yaml: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        let service: ServiceDefinition = serde_yaml::from_str(yaml).map_err(|e| {
            ApicentricError::validation_error(
                format!("Invalid service YAML: {}", e),
                Some("service"),
                Some("Ensure the YAML matches the service definition schema"),
            )
        })?;

        fs::create_dir_all(&state.config.services_directory).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot create services directory: {}", e),
                Some("Check write permissions for the configured directory"),
            )
        })?;

        let file_path = state
            .config
            .services_directory
            .join(format!("{}.yaml", service.name));
        fs::write(&file_path, yaml).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write service file: {}", e),
                Some("Verify disk space and permissions"),
            )
        })?;

        // Apply to running simulator if possible
        if let Err(err) = self.manager.apply_service_yaml(yaml).await {
            state.add_log(format!(
                "Applied YAML locally but failed to load service: {}",
                err
            ));
        }

        // Update state services list
        if let Some(existing) = state.find_service_mut(&service.name) {
            existing.path = file_path.clone();
            existing.mark_running();
        } else {
            let mut info = ServiceInfo::new(
                service.name.clone(),
                file_path.clone(),
                state.config.default_port,
            );
            if let Some(endpoints) = &service.endpoints {
                for endpoint in endpoints {
                    info.endpoints.push(EndpointInfo {
                        method: endpoint.method.clone(),
                        path: endpoint.path.clone(),
                    });
                }
            }
            state.add_service(info);
        }

        state.add_log(format!("Applied AI-generated YAML to {:?}", file_path));
        Ok(())
    }

    // Recording Mode Handlers

    async fn handle_start_recording(
        &self,
        target_url: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        let session = super::models::RecordingSession {
            id: uuid::Uuid::new_v4().to_string(),
            target_url: target_url.to_string(),
            proxy_port: state.config.default_port,
            is_active: true,
            captured_requests: Vec::new(),
        };

        state.recording_session = Some(session);
        state.add_log(format!("Started recording: {}", target_url));
        Ok(())
    }

    async fn handle_stop_recording(&self, state: &mut GuiAppState) -> ApicentricResult<()> {
        if let Some(session) = &mut state.recording_session {
            session.is_active = false;
            state.add_log("Stopped recording".to_string());
        }
        Ok(())
    }

    async fn handle_capture_request(
        &self,
        request: CapturedRequest,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        if let Some(session) = &mut state.recording_session {
            if !session.is_active {
                return Err(ApicentricError::runtime_error(
                    "Recording session is not active",
                    Some("Start a recording session first"),
                ));
            }
            session
                .captured_requests
                .push(super::models::RecordedRequest {
                    method: request.method,
                    url: request.url,
                    headers: request.headers,
                    body: request.body,
                });
            state.add_log("Captured request".to_string());
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                "No active recording session",
                Some("Start recording before capturing requests"),
            ))
        }
    }

    async fn handle_generate_from_recording(
        &self,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        let session = state.recording_session.as_mut().ok_or_else(|| {
            ApicentricError::runtime_error(
                "No recording session to generate from",
                Some("Record traffic before generating a service"),
            )
        })?;

        if session.captured_requests.is_empty() {
            return Err(ApicentricError::validation_error(
                "No captured requests to generate from",
                Some("captured_requests"),
                Some("Capture at least one request"),
            ));
        }

        let mut endpoints: HashMap<(String, String), EndpointDefinition> = HashMap::new();
        for req in &session.captured_requests {
            let key = (req.method.clone(), req.url.clone());
            endpoints.entry(key).or_insert_with(|| EndpointDefinition {
                kind: apicentric::simulator::config::EndpointKind::Http,
                method: req.method.to_uppercase(),
                path: req.url.split('?').next().unwrap_or("/").to_string(),
                header_match: None,
                description: Some("Recorded endpoint".to_string()),
                parameters: None,
                request_body: None,
                responses: {
                    let mut map = HashMap::new();
                    map.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: req.body.clone().unwrap_or_else(|| "{}".to_string()),
                            script: None,
                            headers: None,
                            side_effects: None,
                            schema: None,
                        },
                    );
                    map
                },
                scenarios: None,
                stream: None,
            });
        }

        let def = ServiceDefinition {
            name: "recording_service".to_string(),
            version: None,
            description: Some(format!("Recorded from {}", session.target_url)),
            server: Some(ServerConfig {
                port: Some(session.proxy_port),
                base_path: "/".to_string(),
                proxy_base_url: Some(session.target_url.clone()),
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Some(endpoints.into_values().collect()),
            graphql: None,
            behavior: None,
            twin: None,
        };

        fs::create_dir_all(&state.config.services_directory).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot create services directory: {}", e),
                Some("Check write permissions"),
            )
        })?;
        let file_path = state
            .config
            .services_directory
            .join("recording_service.yaml");
        let yaml = serde_yaml::to_string(&def).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to serialize recorded service: {}", e),
                None::<String>,
            )
        })?;
        fs::write(&file_path, yaml).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write recorded service: {}", e),
                Some("Check path permissions"),
            )
        })?;
        session.is_active = false;
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
            let content = fs::read_to_string(&service.path).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to load service file: {}", e),
                    Some("Ensure the service file exists and is readable"),
                )
            })?;
            state.load_service_in_editor(name.to_string(), content);
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                format!("Service not found: {}", name),
                None::<String>,
            ))
        }
    }

    async fn handle_save_editor_content(&self, state: &mut GuiAppState) -> ApicentricResult<()> {
        if let Some(service_name) = state.editor_state.selected_service.clone() {
            let service = state.find_service(&service_name).cloned().ok_or_else(|| {
                ApicentricError::runtime_error(
                    format!("Service not found: {}", service_name),
                    Some("Reload services before saving"),
                )
            })?;

            // Validate YAML
            serde_yaml::from_str::<ServiceDefinition>(&state.editor_state.content).map_err(
                |e| {
                    ApicentricError::validation_error(
                        format!("Invalid YAML: {}", e),
                        Some("editor"),
                        Some("Correct the YAML before saving"),
                    )
                },
            )?;

            fs::write(&service.path, &state.editor_state.content).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to write service file: {}", e),
                    Some("Check write permissions"),
                )
            })?;

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
            entry.method, entry.path, entry.status_code, entry.duration_ms
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
        let content = fs::read_to_string(path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to read import file: {}", e),
                Some("Check the file path and permissions"),
            )
        })?;

        let service = if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            .unwrap_or(false)
        {
            apicentric::simulator::postman::from_str(&content).map_err(|e| {
                ApicentricError::validation_error(
                    format!("Failed to parse Postman/Insomnia export: {}", e),
                    Some("import"),
                    Some("Ensure the file is a valid collection"),
                )
            })?
        } else {
            serde_yaml::from_str::<ServiceDefinition>(&content).or_else(|_| {
                serde_json::from_str::<ServiceDefinition>(&content).map_err(|e| {
                    ApicentricError::validation_error(
                        format!("Unrecognized import format: {}", e),
                        Some("import"),
                        Some("Provide a Postman/Insomnia JSON or simulator YAML"),
                    )
                })
            })?
        };

        self.persist_service_definition(&service, state)?;
        state.add_log(format!("Imported file: {:?}", path));
        Ok(())
    }

    async fn handle_export_service(
        &self,
        name: &str,
        format: ExportFormat,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        let service = self.load_service_by_name(name, state)?;

        let output_path = match format {
            ExportFormat::Yaml => state
                .config
                .services_directory
                .join(format!("{}.yaml", service.name)),
            ExportFormat::Json => state
                .config
                .services_directory
                .join(format!("{}.json", service.name)),
            ExportFormat::Postman => state
                .config
                .services_directory
                .join(format!("{}.postman.json", service.name)),
        };

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to prepare export directory: {}", e),
                    Some("Check permissions"),
                )
            })?;
        }

        match format {
            ExportFormat::Yaml => {
                let yaml = serde_yaml::to_string(&service).map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to serialize YAML: {}", e),
                        None::<String>,
                    )
                })?;
                fs::write(&output_path, yaml).map_err(|e| {
                    ApicentricError::fs_error(
                        format!("Failed to write YAML export: {}", e),
                        Some("Check disk space"),
                    )
                })?;
            }
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&service).map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to serialize JSON: {}", e),
                        None::<String>,
                    )
                })?;
                fs::write(&output_path, json).map_err(|e| {
                    ApicentricError::fs_error(
                        format!("Failed to write JSON export: {}", e),
                        Some("Check disk space"),
                    )
                })?;
            }
            ExportFormat::Postman => {
                let json = apicentric::simulator::postman::to_string(&service).map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to create Postman export: {}", e),
                        None::<String>,
                    )
                })?;
                fs::write(&output_path, json).map_err(|e| {
                    ApicentricError::fs_error(
                        format!("Failed to write Postman export: {}", e),
                        Some("Check disk space"),
                    )
                })?;
            }
        }

        state.add_log(format!("Exported service: {}", name));
        Ok(())
    }

    async fn handle_batch_import(
        &self,
        paths: &[PathBuf],
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        for path in paths {
            self.handle_import_file(path, state).await?;
        }
        state.add_log(format!("Batch imported {} files", paths.len()));
        Ok(())
    }

    // Code Generation Handlers

    async fn handle_generate_code(
        &self,
        name: &str,
        target: CodeGenTarget,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        let service = self.load_service_by_name(name, state)?;
        let code = match target {
            CodeGenTarget::TypeScript | CodeGenTarget::JavaScript | CodeGenTarget::Python => {
                let spec = apicentric::simulator::openapi::to_openapi(&service);
                serde_json::to_string_pretty(&spec).map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to serialize OpenAPI spec: {}", e),
                        None::<String>,
                    )
                })?
            }
            CodeGenTarget::Go | CodeGenTarget::Rust => {
                serde_yaml::to_string(&service).map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to serialize service definition: {}", e),
                        None::<String>,
                    )
                })?
            }
        };

        state.codegen_state.last_target = Some(format!("{:?}", target));
        state.codegen_state.last_output = Some(code);
        state.add_log(format!("Generated code for: {}", name));
        Ok(())
    }

    async fn handle_copy_to_clipboard(
        &self,
        content: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        state.codegen_state.last_output = Some(content.to_string());
        state.add_log("Copied to clipboard".to_string());
        Ok(())
    }

    async fn handle_save_generated_code(
        &self,
        path: &Path,
        content: &str,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to create directories: {}", e),
                    Some("Check write permissions"),
                )
            })?;
        }
        fs::write(path, content).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to save generated code: {}", e),
                Some("Check disk space"),
            )
        })?;
        state.add_log(format!("Saved generated code to: {:?}", path));
        Ok(())
    }

    // Configuration Handlers

    async fn handle_save_config(&self, state: &mut GuiAppState) -> ApicentricResult<()> {
        let path = state.config.services_directory.join("gui_config.yaml");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to prepare config directory: {}", e),
                    Some("Check write permissions"),
                )
            })?;
        }
        let yaml = serde_yaml::to_string(&state.config).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to serialize GUI config: {}", e),
                None::<String>,
            )
        })?;
        fs::write(&path, yaml).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write GUI config: {}", e),
                Some("Check permissions"),
            )
        })?;
        state.add_log("Saved configuration".to_string());
        Ok(())
    }

    async fn handle_load_config(&self, state: &mut GuiAppState) -> ApicentricResult<()> {
        let path = state.config.services_directory.join("gui_config.yaml");
        if !path.exists() {
            state.add_log("No saved configuration found".to_string());
            return Ok(());
        }

        let yaml = fs::read_to_string(&path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to read GUI config: {}", e),
                Some("Check permissions"),
            )
        })?;
        let cfg: super::models::GuiConfig = serde_yaml::from_str(&yaml).map_err(|e| {
            ApicentricError::validation_error(
                format!("Invalid GUI config: {}", e),
                Some("gui_config"),
                Some("Regenerate configuration using the GUI"),
            )
        })?;
        state.config = cfg;
        state.add_log("Loaded configuration".to_string());
        Ok(())
    }
}

impl EventHandler {
    fn persist_service_definition(
        &self,
        service: &ServiceDefinition,
        state: &mut GuiAppState,
    ) -> ApicentricResult<()> {
        fs::create_dir_all(&state.config.services_directory).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to prepare services directory: {}", e),
                Some("Check permissions"),
            )
        })?;

        let path = state
            .config
            .services_directory
            .join(format!("{}.yaml", service.name));
        let yaml = serde_yaml::to_string(service).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to serialize service: {}", e),
                None::<String>,
            )
        })?;
        fs::write(&path, yaml).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write service file: {}", e),
                Some("Check permissions"),
            )
        })?;

        if let Some(existing) = state.find_service_mut(&service.name) {
            existing.path = path.clone();
        } else {
            let mut info = ServiceInfo::new(
                service.name.clone(),
                path.clone(),
                state.config.default_port,
            );
            if let Some(endpoints) = &service.endpoints {
                for endpoint in endpoints {
                    info.endpoints.push(EndpointInfo {
                        method: endpoint.method.clone(),
                        path: endpoint.path.clone(),
                    });
                }
            }
            state.add_service(info);
        }

        Ok(())
    }

    fn load_service_by_name(
        &self,
        name: &str,
        state: &GuiAppState,
    ) -> ApicentricResult<ServiceDefinition> {
        let service = state
            .find_service(name)
            .ok_or_else(|| ApicentricError::runtime_error("Service not found", None::<String>))?;
        let content = fs::read_to_string(&service.path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to read service file: {}", e),
                Some("Ensure the service exists"),
            )
        })?;

        serde_yaml::from_str(&content)
            .or_else(|_| serde_json::from_str(&content))
            .map_err(|e| {
                ApicentricError::validation_error(
                    format!("Invalid service definition: {}", e),
                    Some("service"),
                    Some("Correct the service file contents"),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> Arc<ApiSimulatorManager> {
        use apicentric::simulator::config::{PortRange, SimulatorConfig};
        let config = SimulatorConfig {
            enabled: true,
            services_dir: std::path::PathBuf::from("test_services"),
            port_range: PortRange {
                start: 9000,
                end: 9099,
            },
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
        let result = handler
            .handle_message(GuiMessage::ClearLogs, &mut state)
            .await;
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
        let result = handler
            .handle_message(
                GuiMessage::EditorContentChanged(new_content.clone()),
                &mut state,
            )
            .await;

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

        let result = handler
            .handle_message(GuiMessage::UpdateConfig(new_config.clone()), &mut state)
            .await;

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
        let result = handler
            .handle_message(
                GuiMessage::StartService("test-service".to_string()),
                &mut state,
            )
            .await;

        assert!(result.is_ok());

        // Verify service is running
        let service = state.find_service("test-service").unwrap();
        assert!(service.status.is_running());

        // Stop the service
        let result = handler
            .handle_message(
                GuiMessage::StopService("test-service".to_string()),
                &mut state,
            )
            .await;

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
        handler
            .handle_message(
                GuiMessage::StartService("service-1".to_string()),
                &mut state,
            )
            .await
            .unwrap();

        handler
            .handle_message(
                GuiMessage::StartService("service-2".to_string()),
                &mut state,
            )
            .await
            .unwrap();

        // Verify both are running
        assert!(state.find_service("service-1").unwrap().status.is_running());
        assert!(state.find_service("service-2").unwrap().status.is_running());

        // Stop first service
        handler
            .handle_message(GuiMessage::StopService("service-1".to_string()), &mut state)
            .await
            .unwrap();

        // Verify first is stopped, second still running
        assert_eq!(
            state.find_service("service-1").unwrap().status,
            super::super::models::ServiceStatus::Stopped
        );
        assert!(state.find_service("service-2").unwrap().status.is_running());
    }

    #[tokio::test]
    async fn test_service_lifecycle_error_nonexistent() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);

        // Try to start nonexistent service
        let result = handler
            .handle_message(
                GuiMessage::StartService("nonexistent".to_string()),
                &mut state,
            )
            .await;

        assert!(result.is_err());

        // Try to stop nonexistent service
        let result = handler
            .handle_message(
                GuiMessage::StopService("nonexistent".to_string()),
                &mut state,
            )
            .await;

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

        handler
            .handle_message(
                GuiMessage::StartService("test-service".to_string()),
                &mut state,
            )
            .await
            .unwrap();

        // Try to start again
        let result = handler
            .handle_message(
                GuiMessage::StartService("test-service".to_string()),
                &mut state,
            )
            .await;

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
        let result = handler
            .handle_message(
                GuiMessage::StopService("test-service".to_string()),
                &mut state,
            )
            .await;

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
        handler
            .handle_message(
                GuiMessage::ServiceStatusChanged(
                    "test-service".to_string(),
                    super::super::models::ServiceStatus::Starting,
                ),
                &mut state,
            )
            .await
            .unwrap();

        assert_eq!(
            state.find_service("test-service").unwrap().status,
            super::super::models::ServiceStatus::Starting
        );

        handler
            .handle_message(
                GuiMessage::ServiceStatusChanged(
                    "test-service".to_string(),
                    super::super::models::ServiceStatus::Running,
                ),
                &mut state,
            )
            .await
            .unwrap();

        assert!(state
            .find_service("test-service")
            .unwrap()
            .status
            .is_running());
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
        handler
            .handle_message(
                GuiMessage::ServiceStatusChanged(
                    "test-service".to_string(),
                    super::super::models::ServiceStatus::Failed("Port in use".to_string()),
                ),
                &mut state,
            )
            .await
            .unwrap();

        assert!(state
            .find_service("test-service")
            .unwrap()
            .status
            .is_failed());

        // Should be able to restart from failed state
        let result = handler
            .handle_message(
                GuiMessage::StartService("test-service".to_string()),
                &mut state,
            )
            .await;

        assert!(result.is_ok());
        assert!(state
            .find_service("test-service")
            .unwrap()
            .status
            .is_running());
    }

    #[tokio::test]
    async fn test_ai_apply_yaml_persists_file() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        let temp = tempfile::tempdir().unwrap();
        state.config.services_directory = temp.path().to_path_buf();

        let yaml = r#"
name: recorded
server:
  base_path: "/"
endpoints:
  - method: GET
    path: "/hello"
    responses:
      200:
        content_type: application/json
        body: "{}"
"#;

        handler
            .handle_message(GuiMessage::AiApplyYaml(yaml.to_string()), &mut state)
            .await
            .unwrap();

        let expected = temp.path().join("recorded.yaml");
        assert!(expected.exists());
        let saved = std::fs::read_to_string(expected).unwrap();
        assert!(saved.contains("recorded"));
    }

    #[tokio::test]
    async fn test_recording_flow_and_generation() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        let temp = tempfile::tempdir().unwrap();
        state.config.services_directory = temp.path().to_path_buf();

        handler
            .handle_message(
                GuiMessage::StartRecording("http://example.com".into()),
                &mut state,
            )
            .await
            .unwrap();

        handler
            .handle_message(
                GuiMessage::CaptureRequest(super::super::messages::CapturedRequest {
                    method: "GET".into(),
                    url: "http://example.com/api".into(),
                    headers: vec![("Content-Type".into(), "application/json".into())],
                    body: Some("{}".into()),
                }),
                &mut state,
            )
            .await
            .unwrap();

        handler
            .handle_message(GuiMessage::StopRecording, &mut state)
            .await
            .unwrap();

        handler
            .handle_message(GuiMessage::GenerateFromRecording, &mut state)
            .await
            .unwrap();

        assert!(temp.path().join("recording_service.yaml").exists());
        assert!(state.recording_session.is_some());
    }

    #[tokio::test]
    async fn test_editor_load_and_save() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("service.yaml");
        let yaml = "name: svc\nserver:\n  base_path: '/'\nendpoints: []\n";
        std::fs::write(&file_path, yaml).unwrap();

        state.add_service(super::super::models::ServiceInfo::new(
            "svc".into(),
            file_path.clone(),
            8000,
        ));

        handler
            .handle_message(GuiMessage::LoadServiceInEditor("svc".into()), &mut state)
            .await
            .unwrap();

        assert_eq!(state.editor_state.content, yaml);
        state.editor_state.content = yaml.replace("svc", "svc2");

        handler
            .handle_message(GuiMessage::SaveEditorContent, &mut state)
            .await
            .unwrap();

        let saved = std::fs::read_to_string(&file_path).unwrap();
        assert!(saved.contains("svc2"));
        assert!(!state.editor_state.dirty);
    }

    #[tokio::test]
    async fn test_import_export_and_batch() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        let temp = tempfile::tempdir().unwrap();
        state.config.services_directory = temp.path().to_path_buf();

        let import_path = temp.path().join("demo.yaml");
        let yaml = r#"
name: demo
server:
  base_path: "/"
endpoints:
  - method: GET
    path: "/ping"
    responses:
      200:
        content_type: application/json
        body: "{}"
"#;
        std::fs::write(&import_path, yaml).unwrap();

        handler
            .handle_message(GuiMessage::ImportFile(import_path.clone()), &mut state)
            .await
            .unwrap();

        assert_eq!(state.services.len(), 1);

        handler
            .handle_message(
                GuiMessage::ExportService(
                    "demo".into(),
                    super::super::messages::ExportFormat::Json,
                ),
                &mut state,
            )
            .await
            .unwrap();

        assert!(temp.path().join("demo.json").exists());

        let another = temp.path().join("demo2.yaml");
        std::fs::write(&another, yaml.replace("demo", "demo2")).unwrap();
        handler
            .handle_message(GuiMessage::BatchImport(vec![another.clone()]), &mut state)
            .await
            .unwrap();
        assert_eq!(state.services.len(), 2);
    }

    #[tokio::test]
    async fn test_code_generation_and_save() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        let temp = tempfile::tempdir().unwrap();
        state.config.services_directory = temp.path().to_path_buf();
        let service_path = temp.path().join("demo.yaml");
        let yaml = r#"
name: demo
server:
  base_path: "/"
endpoints:
  - method: GET
    path: "/ping"
    responses:
      200:
        content_type: application/json
        body: "{}"
"#;
        std::fs::write(&service_path, yaml).unwrap();
        state.add_service(super::super::models::ServiceInfo::new(
            "demo".into(),
            service_path.clone(),
            8000,
        ));

        handler
            .handle_message(
                GuiMessage::GenerateCode(
                    "demo".into(),
                    super::super::messages::CodeGenTarget::Python,
                ),
                &mut state,
            )
            .await
            .unwrap();

        assert!(state.codegen_state.last_output.is_some());

        let code_path = temp.path().join("generated.py");
        handler
            .handle_message(
                GuiMessage::SaveGeneratedCode(code_path.clone(), "print('hi')".into()),
                &mut state,
            )
            .await
            .unwrap();

        assert!(code_path.exists());
    }

    #[tokio::test]
    async fn test_config_save_and_load() {
        let manager = create_test_manager();
        let handler = EventHandler::new(manager);
        let log_receiver = tokio::sync::broadcast::channel(1).1;
        let mut state = GuiAppState::new(log_receiver);
        let temp = tempfile::tempdir().unwrap();
        state.config.services_directory = temp.path().to_path_buf();
        state.config.default_port = 7000;

        handler
            .handle_message(GuiMessage::SaveConfig, &mut state)
            .await
            .unwrap();

        state.config.default_port = 0;
        handler
            .handle_message(GuiMessage::LoadConfig, &mut state)
            .await
            .unwrap();

        assert_eq!(state.config.default_port, 7000);
    }
}
