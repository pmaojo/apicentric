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
use super::tui_render::{render_service_list, render_log_view, render_actions_panel_with_metrics, render_filter_dialog, render_search_dialog, render_help_dialog};
use super::tui_state::{TuiAppState, ViewMode};

/// Launch the enhanced terminal dashboard with service list, logs and actions panes.
///
/// The interface provides:
/// - Real-time service status monitoring
/// - Live request log streaming with filtering
/// - Keyboard-driven navigation and control
/// - Service start/stop management
///
/// The interface exits gracefully when `Ctrl+C` or `q` is pressed.
pub async fn tui_command() -> ApicentricResult<()> {
    // Initialize the simulator manager
    let config = SimulatorConfig::default();
    let manager = Arc::new(ApiSimulatorManager::new(config));

    // Set up terminal
    enable_raw_mode().map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to enable raw mode: {}", e),
            Some("Try running in a different terminal or check terminal permissions")
        )
    })?;
    
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to initialize terminal: {}", e),
            Some("Ensure your terminal supports alternate screen mode")
        )
    })?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to create terminal backend: {}", e),
            Some("Try using a different terminal emulator or check terminal compatibility")
        )
    })?;

    // Run the application
    let res = run_app(&mut terminal, manager).await;

    // Restore terminal
    disable_raw_mode().map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to disable raw mode: {}", e),
            Some("Terminal may be in an inconsistent state. Try closing and reopening your terminal")
        )
    })?;
    
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to restore terminal: {}", e),
            Some("Terminal may be in an inconsistent state. Try closing and reopening your terminal")
        )
    })?;
    
    terminal.show_cursor().map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to show cursor: {}", e),
            Some("Run 'tput cnorm' to restore cursor visibility")
        )
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
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    manager: Arc<ApiSimulatorManager>,
) -> ApicentricResult<()> {
    // Initialize state
    let mut state = TuiAppState::new();
    
    // Subscribe to log events
    let mut log_receiver = manager.subscribe_logs();

    // Initial status update
    update_service_status(&mut state, &manager).await?;

    // Track time for periodic status updates (every 1 second)
    let mut last_status_update = std::time::Instant::now();
    let status_update_interval = Duration::from_secs(1);

    // Performance profiling variables
    let mut input_latencies = Vec::new();
    let mut max_input_latency = Duration::ZERO;

    // Main event loop
    loop {
        let loop_start = std::time::Instant::now();

        // Render UI
        let render_start = std::time::Instant::now();
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

                // Render panels with focus indication
                render_service_list(
                    f,
                    chunks[0],
                    &state.services,
                    state.focused_panel == super::tui_state::FocusedPanel::Services
                );
                render_log_view(
                    f,
                    chunks[1],
                    &state,
                    state.focused_panel == super::tui_state::FocusedPanel::Logs
                );
                // Calculate average input latency for display
                let avg_input_latency = if !input_latencies.is_empty() {
                    let sum: Duration = input_latencies.iter().sum();
                    sum / input_latencies.len() as u32
                } else {
                    Duration::ZERO
                };

                render_actions_panel_with_metrics(
                    f,
                    chunks[2],
                    &state,
                    if avg_input_latency > Duration::ZERO { Some(avg_input_latency) } else { None },
                    if max_input_latency > Duration::ZERO { Some(max_input_latency) } else { None },
                );

                // Render dialogs on top if active
                match state.mode {
                    ViewMode::FilterDialog => render_filter_dialog(f, &state),
                    ViewMode::SearchDialog => render_search_dialog(f, &state),
                    ViewMode::HelpDialog => render_help_dialog(f),
                    ViewMode::Normal => {},
                }
            })
            .map_err(|e| {
                ApicentricError::runtime_error(
                    format!("Render error: {}", e),
                    Some("Terminal size may be too small. Try resizing your terminal window")
                )
            })?;
        let _render_time = render_start.elapsed();

        // Poll for new log entries (non-blocking)
        let poll_start = std::time::Instant::now();
        poll_log_entries(&mut state, &mut log_receiver);
        let _poll_time = poll_start.elapsed();

        // Periodic status update (every 1 second)
        let _status_update_time = if last_status_update.elapsed() >= status_update_interval {
            let update_start = std::time::Instant::now();
            let _ = update_service_status(&mut state, &manager).await;
            last_status_update = std::time::Instant::now();
            update_start.elapsed()
        } else {
            Duration::ZERO
        };

        // Poll for keyboard events with optimized timeout (50ms for <100ms response)
        let event_start = std::time::Instant::now();
        let event_timeout = Duration::from_millis(50); // Reduced from 250ms for better responsiveness
        let mut _input_detected = false;
        if let Some(event) = poll_events(event_timeout)? {
            if let Event::Key(key) = event {
                _input_detected = true;
                let key_press_time = event_start.elapsed();

                // Track input latency (time from event detection to processing)
                input_latencies.push(key_press_time);
                if key_press_time > max_input_latency {
                    max_input_latency = key_press_time;
                }

                let action = handle_key_event(key, &mut state, &manager).await?;

                if action == Action::Quit {
                    break;
                }
            }
        }
        let _event_time = event_start.elapsed();

        // Maintain latency history (keep last 100 measurements)
        if input_latencies.len() > 100 {
            input_latencies.remove(0);
        }

        // Log performance metrics if loop takes >50ms (for debugging)
        let total_loop_time = loop_start.elapsed();
        if total_loop_time > Duration::from_millis(50) {
            // Optional: could add debug logging here if needed
            // For now, just track internally
        }
    }

    Ok(())
}
