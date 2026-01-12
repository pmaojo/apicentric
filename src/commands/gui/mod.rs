// The GUI submodules are optional and only compiled when the `gui` feature is enabled.
// This keeps the GUI code out of builds that don't opt-in to the feature (similar to `p2p`).

#![allow(unused_imports)]

#[cfg(feature = "gui")]
use std::sync::Arc;

#[cfg(feature = "gui")]
use apicentric::simulator::manager::ApiSimulatorManager;

#[cfg(feature = "gui")]
pub mod ai;
#[cfg(feature = "gui")]
pub mod events;
#[cfg(feature = "gui")]
pub mod messages;
#[cfg(feature = "gui")]
pub mod models;
#[cfg(feature = "gui")]
pub mod render;
#[cfg(feature = "gui")]
pub mod state;
#[cfg(feature = "gui")]
pub mod style;

#[cfg(all(feature = "gui", test))]
pub mod test_utils;

#[cfg(feature = "gui")]
pub use messages::*;
#[cfg(feature = "gui")]
pub use render::*;
#[cfg(feature = "gui")]
pub use state::GuiAppState;

/// Launch the egui GUI application
#[cfg(feature = "gui")]
pub async fn gui_command() -> crate::ApicentricResult<()> {
    use apicentric::simulator::config::{PortRange, SimulatorConfig};
    use apicentric::simulator::manager::ApiSimulatorManager;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    // Initialize simulator manager
    let config = SimulatorConfig {
        enabled: true,
        services_dir: std::path::PathBuf::from("services"),
        port_range: PortRange {
            start: 9000,
            end: 9099,
        },
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
    if let Err(e) = events::EventHandler::new(manager.clone())
        .handle_message(messages::GuiMessage::RefreshServices, &mut gui_state)
        .await
    {
        eprintln!("Failed to refresh services: {}", e);
    }

    // Run the egui application
    run_gui_application(gui_state, manager).await
}

/// Run the main egui GUI application
#[cfg(feature = "gui")]
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
    })
    .map_err(|e| {
        crate::ApicentricError::runtime_error(
            format!("GUI application failed: {}", e),
            None::<String>,
        )
    })
}
