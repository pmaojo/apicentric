//! Graphical User Interface module
//!
//! This module is only available when the `gui` feature is enabled.

#![cfg(feature = "gui")]

mod state;
mod render;
mod events;
mod ai;

use apicentric::{ApicentricError, ApicentricResult};
use eframe::egui;
use state::{GuiAppState, RequestLogEntry, LogFilter};
use apicentric::simulator::{manager::ApiSimulatorManager, config::SimulatorConfig, SimulatorStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use ai::AiServiceGenerator;

// Re-export types for external use
pub use state::{ServiceInfo, ServiceStatus, EndpointInfo};

/// Messages for GUI event handling
#[derive(Debug, Clone)]
pub enum GuiMessage {
    // AI Generation (existing)
    AiGenerate(String),
    AiApplyYaml(String),
    AiGenerationComplete(Result<String, String>),
    
    // Service Management
    StartService(String),
    StopService(String),
    RefreshServices,
    ServiceStatusChanged(String, state::ServiceStatus),
    
    // Recording Mode
    StartRecording(String), // target URL
    StopRecording,
    CaptureRequest(CapturedRequest),
    GenerateFromRecording,
    
    // Editor
    LoadServiceInEditor(String), // service name
    SaveEditorContent,
    EditorContentChanged(String),
    
    // Logs
    NewRequestLog(RequestLogEntry),
    ClearLogs,
    FilterLogsBy(LogFilter),
    
    // Import/Export
    ImportFile(std::path::PathBuf),
    ExportService(String, ExportFormat), // service name, format
    BatchImport(Vec<std::path::PathBuf>),
    
    // Code Generation
    GenerateCode(String, CodeGenTarget), // service name, target
    CopyToClipboard(String),
    SaveGeneratedCode(std::path::PathBuf, String),
    
    // Configuration
    UpdateConfig(state::GuiConfig),
    SaveConfig,
    LoadConfig,
}

/// Captured request data for recording mode
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
    pub response_status: u16,
    pub response_body: String,
}



/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Yaml,
    Postman,
    OpenApi,
    WireMock,
}

/// Code generation target
#[derive(Debug, Clone)]
pub enum CodeGenTarget {
    TypeScript,
    ReactQuery,
    AxiosClient,
}

/// Handle AI generation asynchronously
///
/// This function creates an AI generator from configuration and generates
/// a service definition from the provided prompt. Results are sent back
/// through the provided channel.
///
/// # Arguments
///
/// * `prompt` - The user's prompt describing the desired service
/// * `tx` - Channel sender for returning the result
async fn handle_ai_generate(prompt: String, tx: mpsc::Sender<ApicentricResult<String>>) {
    let result = async {
        // Create generator from configuration
        let generator = AiServiceGenerator::from_config(
            std::path::Path::new("apicentric.json")
        )?;

        // Generate service definition
        generator.generate_from_prompt(&prompt).await
    }
    .await;
    
    // Send result back through channel
    tx.send(result).await.ok();
}

struct ApicentricGuiApp {
    state: GuiAppState,
    manager: Arc<ApiSimulatorManager>,
    event_handler: events::EventHandler,
    status_receiver: mpsc::Receiver<SimulatorStatus>,
    message_sender: mpsc::Sender<GuiMessage>,
    message_receiver: mpsc::Receiver<GuiMessage>,
    ai_result_sender: mpsc::Sender<ApicentricResult<String>>,
    ai_result_receiver: mpsc::Receiver<ApicentricResult<String>>,
}

impl ApicentricGuiApp {
    fn new(_cc: &eframe::CreationContext<'_>, manager: Arc<ApiSimulatorManager>) -> Self {
        let (status_tx, status_rx) = mpsc::channel(1);
        let log_receiver = manager.subscribe_logs();

        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            loop {
                let status = manager_clone.get_status().await;
                if status_tx.send(status).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        let (message_sender, message_receiver) = mpsc::channel(1);
        let (ai_result_sender, ai_result_receiver) = mpsc::channel(1);

        let event_handler = events::EventHandler::new(Arc::clone(&manager));

        Self {
            state: GuiAppState::new(log_receiver),
            manager,
            event_handler,
            status_receiver: status_rx,
            message_sender,
            message_receiver,
            ai_result_sender,
            ai_result_receiver,
        }
    }
}

impl eframe::App for ApicentricGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(status) = self.status_receiver.try_recv() {
            // Update services with enhanced ServiceInfo structure
            self.state.services.clear();
            for active_service in status.active_services {
                let service_info = state::ServiceInfo {
                    name: active_service.name.clone(),
                    path: std::path::PathBuf::from(format!("services/{}.yaml", active_service.name)),
                    status: state::ServiceStatus::Running,
                    port: active_service.port,
                    endpoints: vec![], // TODO: Extract from service definition
                };
                self.state.add_service(service_info);
            }
        }

        while let Ok(message) = self.message_receiver.try_recv() {
            // Handle AI generation specially since it needs async spawn
            if let GuiMessage::AiGenerate(prompt) = &message {
                self.state.start_ai_generation();
                let tx = self.ai_result_sender.clone();
                let prompt = prompt.clone();
                tokio::spawn(async move {
                    handle_ai_generate(prompt, tx).await;
                });
                continue;
            }

            // Handle other messages using EventHandler
            // Note: This is a simplified synchronous handling for now
            // In a production implementation, we'd use a proper async runtime integration
            match message {
                GuiMessage::ClearLogs => {
                    self.state.clear_logs();
                }
                GuiMessage::EditorContentChanged(content) => {
                    self.state.editor_state.content = content;
                    self.state.mark_editor_dirty();
                }
                GuiMessage::AiGenerationComplete(result) => {
                    match result {
                        Ok(yaml) => self.state.complete_ai_generation(yaml),
                        Err(e) => self.state.fail_ai_generation(e),
                    }
                }
                GuiMessage::UpdateConfig(config) => {
                    self.state.config = config;
                }
                GuiMessage::ServiceStatusChanged(name, status) => {
                    self.state.update_service_status(&name, status);
                }
                // For messages that need async operations, we'll handle them via EventHandler
                // in future iterations when we have proper async integration
                _ => {
                    // Log that message was received but not yet fully handled
                    self.state.add_log(format!("Received message: {:?}", message));
                }
            }
        }

        if let Ok(result) = self.ai_result_receiver.try_recv() {
            match result {
                Ok(yaml) => self.state.complete_ai_generation(yaml),
                Err(e) => {
                    self.state.fail_ai_generation(format!("AI Error: {}", e));
                    self.state.add_log(format!("AI Error: {}", e));
                }
            }
        }

        // Update logs from the manager - convert simulator logs to GUI logs
        while let Ok(sim_log) = self.state.log_receiver.try_recv() {
            // Convert simulator log to GUI log entry
            let gui_log = RequestLogEntry::from_simulator_log(&sim_log);
            
            // Add to structured request logs
            self.state.add_request_log(gui_log.clone());
            
            // Also add to string logs for backward compatibility
            let log = format!("[{}] {} {} {} - {}ms",
                sim_log.timestamp.format("%H:%M:%S"),
                sim_log.method,
                sim_log.path,
                sim_log.status,
                0 // Simulator doesn't track duration yet
            );
            self.state.add_log(log);
        }

        ctx.request_repaint_after(Duration::from_millis(500));

        render::render(ctx, &mut self.state, &self.manager, &self.message_sender);
    }
}

/// Launch the graphical user interface.
pub async fn gui_command() -> ApicentricResult<()> {
    // Initialize the simulator manager with proper config
    let config = SimulatorConfig {
        enabled: true,
        services_dir: std::path::PathBuf::from("services"),
        port_range: apicentric::simulator::config::PortRange { start: 9000, end: 9099 },
        db_path: std::path::PathBuf::from("apicentric.db"),
        admin_port: None,
        global_behavior: None,
    };
    let manager = Arc::new(ApiSimulatorManager::new(config));

    // Create a dummy service file so the simulator can start
    std::fs::create_dir_all("services").unwrap_or_default();
    let dummy_service = r#"
name: dummy
version: "1.0"
description: "Dummy service for GUI"
server:
  port: 8080
  base_path: "/api"
endpoints:
  - method: GET
    path: "/health"
    responses:
      200:
        content_type: "application/json"
        body: '{"status": "ok"}'
"#;
    std::fs::write("services/dummy.yaml", dummy_service).unwrap_or_default();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Apicentric GUI",
        native_options,
        Box::new(|cc| Box::new(ApicentricGuiApp::new(cc, manager))),
    )
    .map_err(|e| ApicentricError::runtime_error(format!("GUI Error: {}", e), None::<String>))?;

    Ok(())
}
