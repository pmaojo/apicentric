//! Event handling and status updates for the TUI

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedSender;

use apicentric::simulator::{log::RequestLogEntry, manager::ApiSimulatorManager, manager::TestResult};
use apicentric::ApicentricResult;

use super::tui_state::{FocusedPanel, TuiAppState, ViewMode};
use std::time::{SystemTime, UNIX_EPOCH}; // For random port generation

/// Messages passed from async tasks to the main TUI loop
#[derive(Debug)]
pub enum TuiMessage {
    EndpointTestCompleted(Option<TestResult>),
}

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
    state
        .services
        .update_from_service_info(status.active_services);
    state.services.update_stats_from_logs(&state.logs.entries);
    Ok(())
}

/// Poll for new log entries (non-blocking)
pub fn poll_log_entries(
    state: &mut TuiAppState,
    log_receiver: &mut broadcast::Receiver<RequestLogEntry>,
) {
    // Try to receive up to 10 entries per poll to avoid blocking too long
    for _ in 0..10 {
        match log_receiver.try_recv() {
            Ok(entry) => {
                // Update dashboard metrics (retro telemetry)
                state.dashboard.record_request(entry.service.clone());
                state.logs.add_entry(entry);
            }
            Err(_) => break, // Empty or lagging
        }
    }

    // Update service statistics based on logs
    state.services.update_stats_from_logs(&state.logs.entries);
}

/// Handle keyboard events
pub async fn handle_key_event(
    key: event::KeyEvent,
    state: &mut TuiAppState,
    manager: &Arc<ApiSimulatorManager>,
    tx: &UnboundedSender<TuiMessage>,
) -> ApicentricResult<Action> {
    match state.mode {
        ViewMode::Normal => handle_normal_mode_key(key, state, manager).await,
        ViewMode::FilterDialog => handle_filter_dialog_key(key, state),
        ViewMode::SearchDialog => handle_search_dialog_key(key, state),
        ViewMode::HelpDialog => handle_help_dialog_key(key, state),
        ViewMode::MarketplaceDialog => handle_marketplace_dialog_key(key, state, manager).await,
        ViewMode::ConfigView => handle_config_view_key(key, state),
        ViewMode::EndpointExplorer => handle_endpoint_explorer_key(key, state, manager, tx).await,
    }
}

/// Handle keys in endpoint explorer mode
async fn handle_endpoint_explorer_key(
    key: event::KeyEvent,
    state: &mut TuiAppState,
    manager: &Arc<ApiSimulatorManager>,
    tx: &UnboundedSender<TuiMessage>,
) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('e') => {
            state.mode = ViewMode::Normal;
            state.endpoint_explorer.last_test_result = None;
            Ok(Action::Continue)
        }
        KeyCode::Up => {
            if !state.endpoint_explorer.endpoints.is_empty() {
                state.endpoint_explorer.selected = 
                    state.endpoint_explorer.selected.saturating_sub(1);
                 // Clear result on selection change
                 state.endpoint_explorer.last_test_result = None;
            }
            Ok(Action::Continue)
        }
        KeyCode::Down => {
             if !state.endpoint_explorer.endpoints.is_empty() {
                let max = state.endpoint_explorer.endpoints.len().saturating_sub(1);
                state.endpoint_explorer.selected = 
                    (state.endpoint_explorer.selected + 1).min(max);
                // Clear result on selection change
                state.endpoint_explorer.last_test_result = None;
            }
            Ok(Action::Continue)
        }
        KeyCode::Enter | KeyCode::Char('t') => {
            // Trigger test
             if let Some(service) = state.services.items.get(state.services.selected) {
                 if let Some(endpoint) = state.endpoint_explorer.endpoints.get(state.endpoint_explorer.selected) {
                    state.endpoint_explorer.is_testing = true;
                    let port = service.port;
                    let method = endpoint.method.clone();
                    let path = endpoint.path.clone();
                    let manager = manager.clone();
                    let tx = tx.clone();

                    // Spawn background task
                    tokio::spawn(async move {
                        let result = manager.test_endpoint(port, &method, &path).await.ok();
                        let _ = tx.send(TuiMessage::EndpointTestCompleted(result));
                    });
                 }
             }
            Ok(Action::Continue)
        }
        _ => Ok(Action::Continue),
    }
}

/// Handle keys in config view mode
fn handle_config_view_key(key: event::KeyEvent, state: &mut TuiAppState) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
            state.mode = ViewMode::Normal;
            Ok(Action::Continue)
        }
        KeyCode::Up => {
                state.config_view.scroll = state.config_view.scroll.saturating_sub(1);
                Ok(Action::Continue)
        }
        KeyCode::Down => {
                state.config_view.scroll = state.config_view.scroll.saturating_add(1);
                Ok(Action::Continue)
        }
            _ => Ok(Action::Continue),
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
                        state.set_error(format!(
                            "Failed to toggle service '{}': {}",
                            service_name, e
                        ));
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

        // Open endpoint explorer
        KeyCode::Char('e') => {
            // View endpoints of selected service
            if let Some(service) = state
                .services
                .items
                .get(state.services.selected)
            {
                if let Some(endpoints) = manager.get_service_endpoints(&service.name).await {
                    // Initialize endpoint explorer
                    state.endpoint_explorer.endpoints = endpoints;
                    state.endpoint_explorer.selected = 0;
                    state.endpoint_explorer.scroll = 0;
                    state.endpoint_explorer.last_test_result = None;
                    state.endpoint_explorer.is_testing = false;
                    state.mode = ViewMode::EndpointExplorer;
                } else {
                     state.set_error("Service has no endpoints defined".to_string());
                }
            }
            Ok(Action::Continue)
        }

        // Open config view
        KeyCode::Char('v') => {
            // View configuration of selected service
            if let Some(service) = state
                .services
                .items
                .get(state.services.selected)
            {
                if let Some(config) = manager.get_service_config(&service.name).await {
                        // Initialize config view
                    state.config_view.content = config;
                    state.config_view.scroll = 0;
                    state.mode = ViewMode::ConfigView;
                }
            }
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
            state
                .input
                .setup("Search Logs".to_string(), "Enter search term".to_string());
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

        // Open marketplace dialog
        KeyCode::Char('m') => {
            state.mode = ViewMode::MarketplaceDialog;
            state.clear_messages();
            Ok(Action::Continue)
        }

        // Toggle dashboard
        KeyCode::Char('d') => {
            state.dashboard.toggle();
            if state.dashboard.active {
                state.set_status("Switched to Dashboard view".to_string());
            } else {
                state.set_status("Switched to Logs view".to_string());
            }
            Ok(Action::Continue)
        }

        _ => Ok(Action::Continue),
    }
}

/// Handle keys in filter dialog mode
fn handle_filter_dialog_key(
    key: event::KeyEvent,
    state: &mut TuiAppState,
) -> ApicentricResult<Action> {
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
fn handle_search_dialog_key(
    key: event::KeyEvent,
    state: &mut TuiAppState,
) -> ApicentricResult<Action> {
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
fn handle_help_dialog_key(
    key: event::KeyEvent,
    state: &mut TuiAppState,
) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
            state.mode = ViewMode::Normal;
            Ok(Action::Continue)
        }
        _ => Ok(Action::Continue),
    }
}

/// Handle keys in marketplace dialog mode
async fn handle_marketplace_dialog_key(
    key: event::KeyEvent,
    state: &mut TuiAppState,
    _manager: &Arc<ApiSimulatorManager>,
) -> ApicentricResult<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('m') => {
            state.mode = ViewMode::Normal;
            Ok(Action::Continue)
        }
        KeyCode::Up => {
            state.marketplace.select_previous();
            Ok(Action::Continue)
        }
        KeyCode::Down => {
            state.marketplace.select_next();
            Ok(Action::Continue)
        }
        KeyCode::Enter => {
            // Install selected marketplace item
            if let Some(item) = state.marketplace.selected_item() {
                let item_name = item.name.clone();
                let item_id = item.id.clone();
                let definition_url = item.definition_url.clone();
                
                state.set_loading(true);
                state.mode = ViewMode::Normal;
                state.set_status(format!("Installing '{}'...", item_name));
                
                // Download the YAML definition
                match download_marketplace_item(&item_id, &definition_url).await {
                    Ok(file_path) => {
                        // Reload services to pick up the new file
                        if let Err(e) = update_service_status(state, _manager).await {
                            state.set_error(format!("Installed but failed to refresh: {}", e));
                        } else {
                            state.set_status(format!("'{}' installed to {}", item_name, file_path));
                        }
                    }
                    Err(e) => {
                        state.set_error(format!("Failed to install '{}': {}", item_name, e));
                    }
                }
                
                state.set_loading(false);
            }
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
        state.set_status(format!(
            "Filter applied: {}",
            state.logs.filter.description()
        ));
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
            Some("Check directory permissions or try a different location"),
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
                Some("Check disk space and file permissions"),
            )
        })?;
    }

    state.set_status(format!("Logs saved to {}", filename));
    Ok(())
}

/// Poll for keyboard events with timeout
#[allow(dead_code)]
pub fn poll_events(timeout: Duration) -> ApicentricResult<Option<Event>> {
    if event::poll(timeout).map_err(|e| {
        apicentric::ApicentricError::runtime_error(
            format!("Event poll failed: {}", e),
            Some("Terminal input may be unavailable. Try restarting the TUI"),
        )
    })? {
        let event = event::read().map_err(|e| {
            apicentric::ApicentricError::runtime_error(
                format!("Event read failed: {}", e),
                Some("Terminal input may be unavailable. Try restarting the TUI"),
            )
        })?;
        Ok(Some(event))
    } else {
        Ok(None)
    }
}

/// Download a marketplace item and save to services directory
async fn download_marketplace_item(item_id: &str, url: &str) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    let services_dir = Path::new("services");
    
    // Create services directory if it doesn't exist
    if !services_dir.exists() {
        fs::create_dir_all(services_dir)
            .map_err(|e| format!("Failed to create services directory: {}", e))?;
    }
    
    let file_path = services_dir.join(format!("{}.yaml", item_id));
    
    // Check if already installed - for now, we'll allow overwriting or just return success
    if file_path.exists() {
        // If it exists, we just return the path so the simulator can try loading it
        return Ok(file_path.to_string_lossy().to_string());
    }
    
    // Download the YAML content
    // Note: For URLs that are local examples, we try to copy from examples/
    if url.contains("pmaojo/apicentric") && url.contains("/examples/") {
        // Extract the local path from the URL
        let local_path = if url.contains("iot/") {
            format!("examples/iot/{}.yaml", item_id)
        } else {
            format!("examples/{}.yaml", item_id)
        };
        
        if Path::new(&local_path).exists() {
            fs::copy(&local_path, &file_path)
                .map_err(|e| format!("Failed to copy from examples: {}", e))?;
            return Ok(file_path.to_string_lossy().to_string());
        }
    }
    
    // For external URLs, use reqwest to download
    #[cfg(feature = "reqwest")]
    {
        // Add user agent
        let client = reqwest::Client::builder()
            .user_agent("Apicentric-TUI/0.3.1")
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let response = client.get(url).send().await.map_err(|e| format!("Failed to request URL: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()));
        }
        
        let content = response.text().await.map_err(|e| format!("Failed to read content: {}", e))?;

        // Parse YAML/JSON to inject random port
        // This prevents conflicts if multiple services default to 8080
        let mut final_content = content.clone();
        if let Ok(mut yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            // Skip port injection for Digital Twins
            if yaml.get("twin").is_none() {
                // Generate random port (8000-9000)
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                let port = 8000 + (now % 1000) as u16;

                // Helper to insert port
                if let Some(mapping) = yaml.as_mapping_mut() {
                    let port_val = serde_yaml::Value::Number(serde_yaml::Number::from(port));
                    let base_path_val = serde_yaml::Value::String("/api".to_string());
                    
                    let server_key = serde_yaml::Value::String("server".to_string());
                    
                    if let Some(server) = mapping.get_mut(&server_key) {
                        if let Some(server_map) = server.as_mapping_mut() {
                            server_map.insert(serde_yaml::Value::String("port".to_string()), port_val);
                        }
                    } else {
                        let mut server_map = serde_yaml::Mapping::new();
                        server_map.insert(serde_yaml::Value::String("port".to_string()), port_val);
                        server_map.insert(serde_yaml::Value::String("base_path".to_string()), base_path_val);
                        mapping.insert(server_key, serde_yaml::Value::Mapping(server_map));
                    }
                    
                    if let Ok(modified) = serde_yaml::to_string(&yaml) {
                        final_content = modified;
                    }
                }
            }
        }

        // Save to file
        fs::write(&file_path, final_content).map_err(|e| format!("Failed to save service: {}", e))?;
        Ok(file_path.to_string_lossy().to_string())
    }
    
    #[cfg(not(feature = "reqwest"))]
    Err("HTTP client (reqwest) is not enabled".to_string())
}
