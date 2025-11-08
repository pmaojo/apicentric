//! TUI state management structures for the enhanced terminal interface
//! 
//! This module is only available when the `tui` feature is enabled.

#![cfg(feature = "tui")]

use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};

use apicentric::simulator::log::RequestLogEntry;
use apicentric::simulator::ServiceInfo;

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
    /// Loading indicator for async operations
    pub is_loading: bool,
    /// Currently focused panel
    pub focused_panel: FocusedPanel,
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
            is_loading: false,
            focused_panel: FocusedPanel::Services,
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

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Switch to the next panel
    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Services => FocusedPanel::Logs,
            FocusedPanel::Logs => FocusedPanel::Services,
        };
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

/// Which panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    /// Services panel is focused
    Services,
    /// Logs panel is focused
    Logs,
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
        let existing_stats: HashMap<String, (usize, Option<DateTime<Utc>>)> =
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use apicentric::simulator::log::RequestLogEntry;

    #[test]
    fn test_log_filter_matches_method() {
        let mut filter = LogFilter::new();
        filter.method = Some("GET".to_string());

        let entry = RequestLogEntry {
            timestamp: Utc::now(),
            service: "test-service".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            status: 200,
        };

        assert!(filter.matches(&entry));

        let entry_post = RequestLogEntry {
            timestamp: Utc::now(),
            service: "test-service".to_string(),
            endpoint: None,
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            status: 200,
        };

        assert!(!filter.matches(&entry_post));
    }

    #[test]
    fn test_log_filter_matches_status() {
        let mut filter = LogFilter::new();
        filter.status = Some(200);

        let entry = RequestLogEntry {
            timestamp: Utc::now(),
            service: "test-service".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            status: 200,
        };

        assert!(filter.matches(&entry));

        let entry_404 = RequestLogEntry {
            timestamp: Utc::now(),
            service: "test-service".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            status: 404,
        };

        assert!(!filter.matches(&entry_404));
    }

    #[test]
    fn test_log_filter_matches_service() {
        let mut filter = LogFilter::new();
        filter.service = Some("api-service".to_string());

        let entry = RequestLogEntry {
            timestamp: Utc::now(),
            service: "api-service".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            status: 200,
        };

        assert!(filter.matches(&entry));

        let entry_other = RequestLogEntry {
            timestamp: Utc::now(),
            service: "other-service".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            status: 200,
        };

        assert!(!filter.matches(&entry_other));
    }

    #[test]
    fn test_log_filter_matches_combined() {
        let mut filter = LogFilter::new();
        filter.method = Some("POST".to_string());
        filter.status = Some(201);
        filter.service = Some("user-service".to_string());

        let entry_match = RequestLogEntry {
            timestamp: Utc::now(),
            service: "user-service".to_string(),
            endpoint: None,
            method: "POST".to_string(),
            path: "/api/users".to_string(),
            status: 201,
        };

        assert!(filter.matches(&entry_match));

        let entry_wrong_method = RequestLogEntry {
            timestamp: Utc::now(),
            service: "user-service".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            status: 201,
        };

        assert!(!filter.matches(&entry_wrong_method));
    }

    #[test]
    fn test_log_filter_is_active() {
        let mut filter = LogFilter::new();
        assert!(!filter.is_active());

        filter.method = Some("GET".to_string());
        assert!(filter.is_active());

        filter.clear();
        assert!(!filter.is_active());

        filter.status = Some(200);
        assert!(filter.is_active());
    }

    #[test]
    fn test_log_filter_description() {
        let mut filter = LogFilter::new();
        assert_eq!(filter.description(), "No filters");

        filter.method = Some("GET".to_string());
        assert_eq!(filter.description(), "Method: GET");

        filter.status = Some(200);
        assert_eq!(filter.description(), "Method: GET, Status: 200");

        filter.service = Some("api".to_string());
        assert_eq!(filter.description(), "Method: GET, Status: 200, Service: api");

        filter.clear();
        assert_eq!(filter.description(), "No filters");
    }

    #[test]
    fn test_log_view_filtered_entries() {
        let mut log_view = LogViewState::new();

        // Add some test entries
        log_view.add_entry(RequestLogEntry {
            timestamp: Utc::now(),
            service: "api".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/users".to_string(),
            status: 200,
        });

        log_view.add_entry(RequestLogEntry {
            timestamp: Utc::now(),
            service: "api".to_string(),
            endpoint: None,
            method: "POST".to_string(),
            path: "/users".to_string(),
            status: 201,
        });

        log_view.add_entry(RequestLogEntry {
            timestamp: Utc::now(),
            service: "auth".to_string(),
            endpoint: None,
            method: "GET".to_string(),
            path: "/login".to_string(),
            status: 200,
        });

        // No filter - should return all entries
        assert_eq!(log_view.filtered_entries().len(), 3);

        // Filter by method
        log_view.filter.method = Some("GET".to_string());
        assert_eq!(log_view.filtered_entries().len(), 2);

        // Filter by service
        log_view.filter.clear();
        log_view.filter.service = Some("api".to_string());
        assert_eq!(log_view.filtered_entries().len(), 2);

        // Filter by status
        log_view.filter.clear();
        log_view.filter.status = Some(201);
        assert_eq!(log_view.filtered_entries().len(), 1);

        // Combined filter
        log_view.filter.clear();
        log_view.filter.method = Some("GET".to_string());
        log_view.filter.service = Some("api".to_string());
        assert_eq!(log_view.filtered_entries().len(), 1);
    }
}
