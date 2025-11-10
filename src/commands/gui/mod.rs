//! Graphical User Interface module
//!
//! This module is only available when the `gui` feature is enabled.

#![cfg(feature = "gui")]

mod state;
mod render;

use apicentric::{ApicentricError, ApicentricResult};
use eframe::egui;
use state::GuiAppState;
use apicentric::simulator::{manager::ApiSimulatorManager, config::SimulatorConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct ApicentricGuiApp {
    state: GuiAppState,
    manager: Arc<ApiSimulatorManager>,
    last_update: Instant,
}

impl ApicentricGuiApp {
    fn new(_cc: &eframe::CreationContext<'_>, manager: Arc<ApiSimulatorManager>) -> Self {
        Self {
            state: GuiAppState::new(),
            manager,
            last_update: Instant::now(),
        }
    }
}

impl eframe::App for ApicentricGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_update.elapsed() > Duration::from_secs(1) {
            let manager = Arc::clone(&self.manager);
            let status = tokio::runtime::Handle::current().block_on(async {
                manager.get_status().await
            });
            self.state.services = status.active_services.into_iter().map(|s| s.name).collect();
            // TODO: Get logs from the manager
            self.last_update = Instant::now();
        }
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
