//! Terminal User Interface module
//! 
//! This module is only available when the `tui` feature is enabled.

#![cfg(feature = "tui")]

//! Enhanced Terminal User Interface for Apicentric
//!
//! Provides an interactive dashboard for managing services and viewing logs in real-time.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use apicentric::{ApicentricError, ApicentricResult};
use apicentric::simulator::{manager::ApiSimulatorManager, config::SimulatorConfig};

use super::tui_events::{poll_events, handle_key_event, update_service_status, poll_log_entries, Action};
use super::tui_render::{render_service_list, render_log_view, render_actions_panel};
use super::tui_state::TuiAppState;

/// Launch the enhanced terminal dashboard with service list, logs and actions panes.
///
/// The interface provides:
/// - Real-time service status monitoring
/// - Live request log streaming with filtering
/// - Keyboard-driven navigation and control
/// - Service start/stop management
///
/// The interface exits gracefully when `Ctrl+C` or `q` is pressed.
pub fn tui_command() -> ApicentricResult<()> {
    // Initialize the simulator manager
    let config = SimulatorConfig::default();
    let manager = Arc::new(ApiSimulatorManager::new(config));

    // Set up terminal
    enable_raw_mode().map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to enable raw mode: {}", e), None::<String>)
    })?;
    
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to initialize terminal: {}", e),
            None::<String>,
        )
    })?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to create terminal backend: {}", e),
            None::<String>,
        )
    })?;

    // Run the application
    let res = run_app(&mut terminal, manager);

    // Restore terminal
    disable_raw_mode().map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to disable raw mode: {}", e), None::<String>)
    })?;
    
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to restore terminal: {}", e), None::<String>)
    })?;
    
    terminal.show_cursor().map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to show cursor: {}", e), None::<String>)
    })?;

    res
}

/// Main application loop
///
/// Implements real-time service status updates by polling the ApiSimulatorManager
/// every 1 second. The status update mechanism:
/// - Fetches current service list from manager.get_status()
/// - Updates TuiState.services with latest information
/// - Handles service additions and removals automatically
/// - Preserves request statistics for existing services
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    manager: Arc<ApiSimulatorManager>,
) -> ApicentricResult<()> {
    // Initialize state
    let mut state = TuiAppState::new();
    
    // Subscribe to log events
    let mut log_receiver = manager.subscribe_logs();
    
    // Create a tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to create tokio runtime: {}", e),
            None::<String>,
        )
    })?;

    // Initial status update
    rt.block_on(async {
        update_service_status(&mut state, &manager).await
    })?;

    // Track time for periodic status updates (every 1 second)
    let mut last_status_update = std::time::Instant::now();
    let status_update_interval = Duration::from_secs(1);

    // Main event loop
    loop {
        // Render UI
        terminal
            .draw(|f| {
                let size = f.size();
                
                // Create three-panel layout
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(25),  // Services
                        Constraint::Percentage(50),  // Logs
                        Constraint::Percentage(25),  // Actions
                    ])
                    .split(size);

                // Render panels
                render_service_list(f, chunks[0], &state.services);
                render_log_view(f, chunks[1], &state);
                render_actions_panel(f, chunks[2], &state);
            })
            .map_err(|e| {
                ApicentricError::runtime_error(format!("Render error: {}", e), None::<String>)
            })?;

        // Poll for new log entries (non-blocking)
        poll_log_entries(&mut state, &mut log_receiver);

        // Periodic status update (every 1 second)
        if last_status_update.elapsed() >= status_update_interval {
            rt.block_on(async {
                let _ = update_service_status(&mut state, &manager).await;
            });
            last_status_update = std::time::Instant::now();
        }

        // Poll for keyboard events with timeout
        if let Some(event) = poll_events(Duration::from_millis(250))? {
            if let Event::Key(key) = event {
                let action = rt.block_on(async {
                    handle_key_event(key, &mut state, &manager).await
                })?;

                if action == Action::Quit {
                    break;
                }
            }
        }
    }

    Ok(())
}
