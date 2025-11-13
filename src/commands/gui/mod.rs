use std::sync::Arc;
use apicentric::simulator::manager::ApiSimulatorManager;

pub mod models;
pub mod messages;
pub mod state;
pub mod render;
pub mod events;
pub mod style;
pub mod ai;

#[cfg(test)]
pub mod test_utils;

pub use messages::*;
pub use state::GuiAppState;
pub use render::*;

/// Launch the egui GUI application
pub async fn gui_command() -> crate::ApicentricResult<()> {
    use apicentric::simulator::manager::ApiSimulatorManager;
    use apicentric::simulator::config::{SimulatorConfig, PortRange};
    use std::sync::Arc;
    use tokio::sync::broadcast;

    // Initialize simulator manager
    let config = SimulatorConfig {
        enabled: true,
        services_dir: std::path::PathBuf::from("services"),
        port_range: PortRange { start: 9000, end: 9099 },
        db_path: std::path::PathBuf::from("apicentric.db"),
        admin_port: Some(8080),
        global_behavior: None,
    };

    let manager = Arc::new(ApiSimulatorManager::new(config));

    // Create a dummy log receiver for the GUI (simplified implementation)
    let (_tx, log_receiver) = broadcast::channel(100);

    // Initialize GUI state
    let mut gui_state = state::GuiAppState::new(log_receiver);

    // Load initial services
    if let Err(e) = events::EventHandler::new(manager.clone()).handle_message(
        messages::GuiMessage::RefreshServices,
        &mut gui_state
    ).await {
        eprintln!("Failed to refresh services: {}", e);
    }

    // Run the egui application
    run_gui_application(gui_state, manager).await
}

/// Run the main egui GUI application
async fn run_gui_application(
    mut gui_state: state::GuiAppState,
    manager: Arc<ApiSimulatorManager>,
) -> crate::ApicentricResult<()> {
    use eframe::egui;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Apicentric GUI"),
        ..Default::default()
    };

    eframe::run_simple_native("Apicentric GUI", options, move |ctx, _frame| {
        render::render(ctx, &mut gui_state, &manager);
    }).map_err(|e| crate::ApicentricError::runtime_error(
        format!("GUI application failed: {}", e),
        None::<String>
    ))
}