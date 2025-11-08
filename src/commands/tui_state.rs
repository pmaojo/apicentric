//! TUI state management structures for the enhanced terminal interface
//! 
//! This module is only available when the `tui` feature is enabled.

#![cfg(feature = "tui")]

use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};

use crate::simulator::log::RequestLogEntry;
use crate::simulator::ServiceInfo;

/// Main application state for the TUI
#[derive(Debug)]
pub struct TuiAppState {
    /// Current view mode
    pub mode: ViewMode,
    /// Service list state
    pub services: ServiceListState,
    /// Log view state
    pub logs: LogViewState,
    /// Input state for dialogs
    pub input: InputState,
    /// Status message to display
    pub status_message: Option<String>,
    /// Error message to display
    pub error_message: Option<String>,
}

impl TuiAppState {
    /// Create a new TUI application state
    pub fn new() -> Self {
        Self {
            mode: ViewMode::Normal,
            services: ServiceListState::new(),
            logs: LogViewState::new(),
            input: InputState::new(),
            status_message: None,
            error_message: None,
        }
    }

    /// Set a status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.error_message = None;
    }

    /// Set an error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.status_message = None;
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
    }
}

impl Default for TuiAppState {
    fn default() -> Self {
        Self::new()
    }
}

/// View mode for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Normal viewing mode
    Normal,
    /// Filter dialog is open
    FilterDialog,
    /// Search dialog is open
    SearchDialog,
    /// Help dialog is open
    HelpDialog,
}

/// State for the service list panel
#[derive(Debug)]
pub struct ServiceListState {
    /// List of services
    pub items: Vec<ServiceStatus>,
    /// Index of the selected service
    pub selected: usize,
}

impl ServiceListState {
    /// Create a new service list state
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
        }
    }

    /// Update services from ServiceInfo list
    /// Preserves request statistics for existing services
    pub fn update_from_service_info(&mut self, services: Vec<ServiceInfo>) {
        // Create a map of existing services to preserve their statistics
        let existing_stats: std::collections::HashMap<String, (usize, Option<DateTime<Utc>>)> = 
            self.items
                .iter()
                .map(|s| (s.name.clone(), (s.request_count, s.last_request)))
                .collect();

        self.items = services
            .into_iter()
            .map(|info| {
                // Preserve statistics if service already existed
                let (request_count, last_request) = existing_stats
                    .get(&info.name)
                    .copied()
                    .unwrap_or((0, None));

                ServiceStatus {
                    name: info.name,
                    port: info.port,
                    is_running: info.is_running,
                    request_count,
                    last_request,
                }
            })
            .collect();

        // Ensure selected index is valid after service additions/removals
        if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        } else if self.items.is_empty() {
            self.selected = 0;
        }
    }

    /// Get the currently selected service
    pub fn selected_service(&self) -> Option<&ServiceStatus> {
        self.items.get(self.selected)
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Update request statistics from logs
    pub fn update_stats_from_logs(&mut self, logs: &VecDeque<RequestLogEntry>) {
        // Reset counts
        for service in &mut self.items {
            service.request_count = 0;
            service.last_request = None;
        }

        // Count requests per service
        for log in logs {
            if let Some(service) = self.items.iter_mut().find(|s| s.name == log.service) {
                service.request_count += 1;
                if service.last_request.is_none()
                    || service.last_request.as_ref().unwrap() < &log.timestamp
                {
                    service.last_request = Some(log.timestamp);
                }
            }
        }
    }
}

impl Default for ServiceListState {
    fn default() -> Self {
        Self::new()
    }
}

/// Status information for a service
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Port the service is running on
    pub port: u16,
    /// Whether the service is currently running
    pub is_running: bool,
    /// Number of requests received
    pub request_count: usize,
    /// Timestamp of the last request
    pub last_request: Option<DateTime<Utc>>,
}

/// State for the log view panel
#[derive(Debug)]
pub struct LogViewState {
    /// Log entries (bounded queue)
    pub entries: VecDeque<RequestLogEntry>,
    /// Active filter
    pub filter: LogFilter,
    /// Scroll offset
    pub scroll: usize,
    /// Maximum number of entries to keep
    pub max_entries: usize,
}

impl LogViewState {
    /// Create a new log view state
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            filter: LogFilter::default(),
            scroll: 0,
            max_entries: 1000,
        }
    }

    /// Add a new log entry
    pub fn add_entry(&mut self, entry: RequestLogEntry) {
        self.entries.push_front(entry);

        // Trim old entries if we exceed the limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    /// Get filtered log entries
    pub fn filtered_entries(&self) -> Vec<&RequestLogEntry> {
        self.entries
            .iter()
            .filter(|entry| self.filter.matches(entry))
            .collect()
    }

    /// Clear all log entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll = 0;
    }

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize, max: usize) {
        self.scroll = (self.scroll + amount).min(max.saturating_sub(1));
    }

    /// Reset scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }
}

impl Default for LogViewState {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter for log entries
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Filter by HTTP method (e.g., "GET", "POST")
    pub method: Option<String>,
    /// Filter by status code
    pub status: Option<u16>,
    /// Filter by service name
    pub service: Option<String>,
}

impl LogFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a log entry matches the filter
    pub fn matches(&self, entry: &RequestLogEntry) -> bool {
        if let Some(ref method) = self.method {
            if !entry.method.eq_ignore_ascii_case(method) {
                return false;
            }
        }

        if let Some(status) = self.status {
            if entry.status != status {
                return false;
            }
        }

        if let Some(ref service) = self.service {
            if !entry.service.eq_ignore_ascii_case(service) {
                return false;
            }
        }

        true
    }

    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        self.method.is_some() || self.status.is_some() || self.service.is_some()
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.method = None;
        self.status = None;
        self.service = None;
    }

    /// Get a human-readable description of the active filters
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref method) = self.method {
            parts.push(format!("Method: {}", method));
        }

        if let Some(status) = self.status {
            parts.push(format!("Status: {}", status));
        }

        if let Some(ref service) = self.service {
            parts.push(format!("Service: {}", service));
        }

        if parts.is_empty() {
            "No filters".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// State for input dialogs
#[derive(Debug)]
pub struct InputState {
    /// Input buffer
    pub buffer: String,
    /// Cursor position
    pub cursor: usize,
    /// Dialog title
    pub title: String,
    /// Dialog prompt
    pub prompt: String,
}

impl InputState {
    /// Create a new input state
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            title: String::new(),
            prompt: String::new(),
        }
    }

    /// Reset the input state
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.title.clear();
        self.prompt.clear();
    }

    /// Set up for a new input dialog
    pub fn setup(&mut self, title: String, prompt: String) {
        self.reset();
        self.title = title;
        self.prompt = prompt;
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete the character before the cursor
    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    /// Get the current input value
    pub fn value(&self) -> &str {
        &self.buffer
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
