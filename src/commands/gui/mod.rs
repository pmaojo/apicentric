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
            state: GuiAppState::new(),
            manager,
            status_receiver: rx,
        }
    }
}

impl eframe::App for ApicentricGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(status) = self.status_receiver.try_recv() {
            self.state.services = status.active_services.into_iter().map(|s| s.name).collect();
            // TODO: Get logs from the manager
        }

        ctx.request_repaint_after(Duration::from_millis(500));

        render::render(ctx, &mut self.state, &self.manager);
    }
}

/// Launch the graphical user interface.
pub async fn gui_command() -> ApicentricResult<()> {
    // Initialize the simulator manager
    let config = SimulatorConfig::default();
    let manager = Arc::new(ApiSimulatorManager::new(config));

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Apicentric GUI",
        native_options,
        Box::new(|cc| Box::new(ApicentricGuiApp::new(cc, manager))),
    )
    .map_err(|e| ApicentricError::runtime_error(format!("GUI Error: {}", e), None::<String>))?;

    Ok(())
}
