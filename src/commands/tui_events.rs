//! Event handling and status updates for the TUI

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tokio::sync::broadcast;

use apicentric::simulator::{log::RequestLogEntry, manager::ApiSimulatorManager};
use apicentric::ApicentricResult;

use super::tui_state::{TuiAppState, ViewMode, FocusedPanel};

/// Action to take after handling an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Continue running the TUI
    Continue,
    /// Quit the TUI
    Quit,
}

/// Update service status from the manager
pub async fn update_service_status(
    state: &mut TuiAppState,
    manager: &Arc<ApiSimulatorManager>,
) -> ApicentricResult<()> {
    let status = manager.get_status().await;
    state.services.update_from_service_info(status.active_services);
    state.services.update_stats_from_logs(&state.logs.entries);
    Ok(())
}

/// Poll for new log entries (non-blocking)
pub fn poll_log_entries(
    state: &mut TuiAppState,
    log_receiver: &mut broadcast::Receiver<RequestLogEntry>,
) {
    // Try to receive all available log entries without blocking
    while let Ok(entry) = log_receiver.try_recv() {
        state.logs.add_entry(entry);
    }

    // Update service statistics based on logs
    state.services.update_stats_from_logs(&state.logs.entries);
}

/// Handle keyboard events
pub async fn handle_key_event(
    key: event::KeyEvent,
    state: &mut TuiAppState,
    manager: &Arc<ApiSimulatorManager>,
) -> ApicentricResult<Action> {
    match state.mode {
        ViewMode::Normal => handle_normal_mode_key(key, state, manager).await,
        ViewMode::FilterDialog => handle_filter_dialog_key(key, state),
        ViewMode::SearchDialog => handle_search_dialog_key(key, state),
        ViewMode::HelpDialog => handle_help_dialog_key(key, state),
    }
}

/// Handle keys in normal viewing mode
async fn handle_normal_mode_key(
    key: event::KeyEvent,
    state: &mut TuiAppState,
    manager: &Arc<ApiSimulatorManager>,
) -> ApicentricResult<Action> {
    match key.code {
        // Quit
        KeyCode::Char('q') => Ok(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Ok(Action::Quit),

        // Navigation - context-aware based on focused panel
        KeyCode::Up => {
            match state.focused_panel {
                FocusedPanel::Services => {
                    state.services.select_previous();
                }
                FocusedPanel::Logs => {
                    state.logs.scroll_up(1);
                }
            }
            state.clear_messages();
            Ok(Action::Continue)
        }
        KeyCode::Down => {
            match state.focused_panel {
                FocusedPanel::Services => {
                    state.services.select_next();
                }
                FocusedPanel::Logs => {
                    let max = state.logs.filtered_entries().len();
                    state.logs.scroll_down(1, max);
                }
            }
            state.clear_messages();
            Ok(Action::Continue)
        }

        // Service control - toggle start/stop
        KeyCode::Enter => {
            if let Some(service) = state.services.selected_service() {
                let service_name = service.name.clone();
                let is_running = service.is_running;

                // Show loading indicator
                state.set_loading(true);
                state.clear_messages();

                // Toggle service state
                let result = if is_running {
                    manager.stop_service(&service_name).await
                } else {
                    manager.start_service(&service_name).await
                };

                // Clear loading indicator
                state.set_loading(false);

                // Handle result
                match result {
                    Ok(_) => {
                        // Update service status immediately
                        update_service_status(state, manager).await?;
                        
                        let action = if is_running { "stopped" } else { "started" };
                        state.set_status(format!("Service '{}' {}", service_name, action));
                    }
                    Err(e) => {
                        state.set_error(format!("Failed to toggle service '{}': {}", service_name, e));
                    }
                }
            } else {
                state.set_error("No service selected".to_string());
            }
            Ok(Action::Continue)
        }

        // Refresh status
        KeyCode::Char('r') => {
            update_service_status(state, manager).await?;
            state.set_status("Status refreshed".to_string());
            Ok(Action::Continue)
        }

        // Clear logs
        KeyCode::Char('c') => {
            state.logs.clear();
            state.set_status("Logs cleared".to_string());
            Ok(Action::Continue)
        }

        // Save logs
        KeyCode::Char('s') => {
            save_logs_to_file(state)?;
            Ok(Action::Continue)
        }

        // Open filter dialog
        KeyCode::Char('f') => {
            state.mode = ViewMode::FilterDialog;
            state.input.setup(
                "Filter Logs".to_string(),
                "Enter filter (method:GET, status:200, service:api)".to_string(),
            );
            state.clear_messages();
            Ok(Action::Continue)
        }

        // Open search dialog
        KeyCode::Char('/') => {
            state.mode = ViewMode::SearchDialog;
            state.input.setup("Search Logs".to_string(), "Enter search term".to_string());
            state.clear_messages();
            Ok(Action::Continue)
        }

        // Show help
        KeyCode::Char('?') => {
            state.mode = ViewMode::HelpDialog;
            state.clear_messages();
            Ok(Action::Continue)
        }

        // Scroll logs
        KeyCode::PageUp => {
            state.logs.scroll_up(10);
            Ok(Action::Continue)
        }
        KeyCode::PageDown => {
            let max = state.logs.filtered_entries().len();
            state.logs.scroll_down(10, max);
            Ok(Action::Continue)
        }

        // Switch focus between panels
        KeyCode::Tab => {
            state.next_panel();
            state.clear_messages();
            Ok(Action::Continue)
        }

        _ => Ok(Action::Continue),
    }
}

/// Handle keys in filter dialog mode
fn handle_filter_dialog_key(key: event::KeyEvent, state: &mut TuiAppState) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc => {
            state.mode = ViewMode::Normal;
            state.logs.filter.clear();
            state.input.reset();
            state.set_status("Filter cleared".to_string());
            Ok(Action::Continue)
        }
        KeyCode::Enter => {
            // Parse filter input
            let input = state.input.value().to_string();
            parse_and_apply_filter(&input, state);
            state.mode = ViewMode::Normal;
            state.input.reset();
            Ok(Action::Continue)
        }
        KeyCode::Char(c) => {
            state.input.insert_char(c);
            Ok(Action::Continue)
        }
        KeyCode::Backspace => {
            state.input.delete_char();
            Ok(Action::Continue)
        }
        KeyCode::Left => {
            state.input.move_cursor_left();
            Ok(Action::Continue)
        }
        KeyCode::Right => {
            state.input.move_cursor_right();
            Ok(Action::Continue)
        }
        _ => Ok(Action::Continue),
    }
}

/// Handle keys in search dialog mode
fn handle_search_dialog_key(key: event::KeyEvent, state: &mut TuiAppState) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc => {
            state.mode = ViewMode::Normal;
            state.input.reset();
            Ok(Action::Continue)
        }
        KeyCode::Enter => {
            // For now, just close the dialog
            // Search functionality can be implemented later
            state.mode = ViewMode::Normal;
            state.set_status("Search not yet implemented".to_string());
            state.input.reset();
            Ok(Action::Continue)
        }
        KeyCode::Char(c) => {
            state.input.insert_char(c);
            Ok(Action::Continue)
        }
        KeyCode::Backspace => {
            state.input.delete_char();
            Ok(Action::Continue)
        }
        KeyCode::Left => {
            state.input.move_cursor_left();
            Ok(Action::Continue)
        }
        KeyCode::Right => {
            state.input.move_cursor_right();
            Ok(Action::Continue)
        }
        _ => Ok(Action::Continue),
    }
}

/// Handle keys in help dialog mode
fn handle_help_dialog_key(key: event::KeyEvent, state: &mut TuiAppState) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
            state.mode = ViewMode::Normal;
            Ok(Action::Continue)
        }
        _ => Ok(Action::Continue),
    }
}

/// Parse filter input and apply to state
fn parse_and_apply_filter(input: &str, state: &mut TuiAppState) {
    state.logs.filter.clear();

    for part in input.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();

            match key.as_str() {
                "method" => {
                    state.logs.filter.method = Some(value.to_uppercase());
                }
                "status" => {
                    if let Ok(status) = value.parse::<u16>() {
                        state.logs.filter.status = Some(status);
                    }
                }
                "service" => {
                    state.logs.filter.service = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    if state.logs.filter.is_active() {
        state.set_status(format!("Filter applied: {}", state.logs.filter.description()));
    } else {
        state.set_status("No valid filter criteria".to_string());
    }

    // Reset scroll when filter changes
    state.logs.scroll_to_top();
}

/// Save logs to a file with timestamp
fn save_logs_to_file(state: &mut TuiAppState) -> ApicentricResult<()> {
    use chrono::Utc;
    use std::fs::File;
    use std::io::Write;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("apicentric_logs_{}.txt", timestamp);

    let mut file = File::create(&filename).map_err(|e| {
        apicentric::ApicentricError::runtime_error(
            format!("Failed to create log file: {}", e),
            None::<String>,
        )
    })?;

    for entry in state.logs.filtered_entries() {
        writeln!(
            file,
            "{} {} {} {} -> {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.service,
            entry.method,
            entry.path,
            entry.status
        )
        .map_err(|e| {
            apicentric::ApicentricError::runtime_error(
                format!("Failed to write to log file: {}", e),
                None::<String>,
            )
        })?;
    }

    state.set_status(format!("Logs saved to {}", filename));
    Ok(())
}

/// Poll for keyboard events with timeout
pub fn poll_events(timeout: Duration) -> ApicentricResult<Option<Event>> {
    if event::poll(timeout).map_err(|e| {
        apicentric::ApicentricError::runtime_error(format!("Event poll failed: {}", e), None::<String>)
    })? {
        let event = event::read().map_err(|e| {
            apicentric::ApicentricError::runtime_error(format!("Event read failed: {}", e), None::<String>)
        })?;
        Ok(Some(event))
    } else {
        Ok(None)
    }
}
