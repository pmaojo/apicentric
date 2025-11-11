//! Graphical User Interface module
//!
//! This module is only available when the `gui` feature is enabled.

#![cfg(feature = "gui")]

mod state;
mod render;
mod style;

use apicentric::{ApicentricError, ApicentricResult};
use eframe::egui;
use state::GuiAppState;
use apicentric::simulator::{manager::ApiSimulatorManager, config::SimulatorConfig, SimulatorStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use apicentric::ai::{AiProvider, GeminiAiProvider, LocalAiProvider, OpenAiProvider};
use apicentric::config::AiProviderKind;

enum GuiMessage {
    AiGenerate(String),
    AiApplyYaml(String),
}

async fn handle_ai_generate(prompt: String, tx: mpsc::Sender<ApicentricResult<String>>) {
    let result = async {
        let cfg = apicentric::config::load_config(std::path::Path::new("apicentric.json"))?;
        let ai_cfg = match &cfg.ai {
            Some(cfg) => cfg,
        None => {
            return Err(ApicentricError::config_error(
                "AI provider not configured",
                Some("Add an 'ai' section to apicentric.json"),
            ))
        }
    };

    let provider: Box<dyn AiProvider> = match ai_cfg.provider {
        AiProviderKind::Local => {
            let path = ai_cfg
                .model_path
                .clone()
                .unwrap_or_else(|| "model.bin".to_string());
            Box::new(LocalAiProvider::new(path))
        }
        AiProviderKind::Openai => {
            let key = ai_cfg.api_key.clone().ok_or_else(|| {
                ApicentricError::config_error(
                    "OpenAI API key missing",
                    Some("Set ai.api_key in apicentric.json"),
                )
            })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
            Box::new(OpenAiProvider::new(key, model))
        }
        AiProviderKind::Gemini => {
            let key = std::env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| ai_cfg.api_key.clone())
                .ok_or_else(|| {
                    ApicentricError::config_error(
                        "Gemini API key missing",
                        Some(
                            "Set GEMINI_API_KEY environment variable or ai.api_key in apicentric.json",
                        ),
                    )
                })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
            Box::new(GeminiAiProvider::new(key, model))
        }
    };

    provider.generate_yaml(&prompt).await
    }
    .await;
    tx.send(result).await.ok();
}

struct ApicentricGuiApp {
    state: GuiAppState,
    manager: Arc<ApiSimulatorManager>,
    status_receiver: mpsc::Receiver<SimulatorStatus>,
    message_sender: mpsc::Sender<GuiMessage>,
    message_receiver: mpsc::Receiver<GuiMessage>,
    ai_result_sender: mpsc::Sender<ApicentricResult<String>>,
    ai_result_receiver: mpsc::Receiver<ApicentricResult<String>>,
}

impl ApicentricGuiApp {
    fn new(cc: &eframe::CreationContext<'_>, manager: Arc<ApiSimulatorManager>) -> Self {
        cc.egui_ctx.set_style(style::apicentric_style());
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

        Self {
            state: GuiAppState::new(log_receiver),
            manager,
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
            self.state.services = status.active_services.into_iter().map(|s| s.name).collect();
        }

        while let Ok(message) = self.message_receiver.try_recv() {
            match message {
                GuiMessage::AiGenerate(prompt) => {
                    let tx = self.ai_result_sender.clone();
                    tokio::spawn(async move {
                        handle_ai_generate(prompt, tx).await;
                    });
                }
                GuiMessage::AiApplyYaml(yaml) => {
                    let manager = self.manager.clone();
                    tokio::spawn(async move {
                        if let Err(e) = manager.apply_service_yaml(&yaml).await {
                            // Log the error
                        }
                    });
                }
            }
        }

        if let Ok(result) = self.ai_result_receiver.try_recv() {
            match result {
                Ok(yaml) => self.state.ai_generated_yaml = Some(yaml),
                Err(e) => self.state.logs.push(format!("AI Error: {}", e)),
            }
        }

        // Update logs from the manager
        while let Ok(log_entry) = self.state.log_receiver.try_recv() {
            self.state.logs.push(format!("[{}] {} {} {} - {}ms",
                log_entry.timestamp.format("%H:%M:%S"),
                log_entry.method,
                log_entry.path,
                log_entry.status,
                0 // TODO: Add response time tracking
            ));
            // Keep only the last 1000 log entries to prevent memory issues
            if self.state.logs.len() > 1000 {
                self.state.logs.remove(0);
            }
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
