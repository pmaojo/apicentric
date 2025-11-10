//! Graphical User Interface module
//!
//! This module is only available when the `gui` feature is enabled.

#![cfg(feature = "gui")]

mod state;
mod render;

use apicentric::{ApicentricError, ApicentricResult};
use eframe::egui;
use state::GuiAppState;
use apicentric::simulator::{manager::ApiSimulatorManager, config::SimulatorConfig, SimulatorStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

struct ApicentricGuiApp {
    state: GuiAppState,
    manager: Arc<ApiSimulatorManager>,
    status_receiver: mpsc::Receiver<SimulatorStatus>,
}

impl ApicentricGuiApp {
    fn new(_cc: &eframe::CreationContext<'_>, manager: Arc<ApiSimulatorManager>) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let log_receiver = manager.subscribe_logs();

        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            loop {
                let status = manager_clone.get_status().await;
                if tx.send(status).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Self {
            state: GuiAppState::new(log_receiver),
            manager,
            status_receiver: rx,
        }
    }
}

impl eframe::App for ApicentricGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(status) = self.status_receiver.try_recv() {
            self.state.services = status.active_services.into_iter().map(|s| s.name).collect();
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

        render::render(ctx, &mut self.state, &self.manager);
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
