//! Data models for the GUI application
//!
//! This module contains all the data structures used by the GUI.

#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::sync::broadcast;

/// Request log entry for GUI display
#[derive(Debug, Clone, PartialEq)]
pub struct RequestLogEntry {
    pub timestamp: SystemTime,
    pub service_name: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
}

impl RequestLogEntry {
    /// Create a new request log entry
    pub fn new(
        service_name: String,
        method: String,
        path: String,
        status_code: u16,
        duration_ms: u64,
    ) -> Self {
        Self {
            timestamp: SystemTime::now(),
            service_name,
            method,
            path,
            status_code,
            duration_ms,
        }
    }

    /// Create a request log entry with a specific timestamp
    pub fn with_timestamp(
        timestamp: SystemTime,
        service_name: String,
        method: String,
        path: String,
        status_code: u16,
        duration_ms: u64,
    ) -> Self {
        Self {
            timestamp,
            service_name,
            method,
            path,
            status_code,
            duration_ms,
        }
    }

    /// Convert from simulator log entry
    pub fn from_simulator_log(log: &apicentric::simulator::log::RequestLogEntry) -> Self {
        Self {
            timestamp: SystemTime::now(), // Use current time since simulator uses DateTime<Utc>
            service_name: log.service.clone(),
            method: log.method.clone(),
            path: log.path.clone(),
            status_code: log.status,
            duration_ms: 0, // Simulator log doesn't track duration
        }
    }
}

/// Filter for request logs
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LogFilter {
    #[default]
    All,
    Service(String),
    StatusCode(u16),
    Method(String),
}

impl LogFilter {
    /// Check if a log entry matches this filter
    pub fn matches(&self, entry: &RequestLogEntry) -> bool {
        match self {
            LogFilter::All => true,
            LogFilter::Service(name) => entry.service_name == *name,
            LogFilter::StatusCode(code) => entry.status_code == *code,
            LogFilter::Method(method) => entry.method == *method,
        }
    }
}

/// Information about a service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub path: PathBuf,
    pub status: ServiceStatus,
    pub port: u16,
    pub endpoints: Vec<EndpointInfo>,
}

impl ServiceInfo {
    /// Create a new ServiceInfo
    pub fn new(name: String, path: PathBuf, port: u16) -> Self {
        Self {
            name,
            path,
            status: ServiceStatus::Stopped,
            port,
            endpoints: Vec::new(),
        }
    }

    /// Transition to starting state
    pub fn start(&mut self) -> Result<(), String> {
        if !self.status.can_start() {
            return Err(format!(
                "Cannot start service in {} state",
                self.status.display_string()
            ));
        }
        self.status = ServiceStatus::Starting;
        Ok(())
    }

    /// Transition to running state
    pub fn mark_running(&mut self) {
        self.status = ServiceStatus::Running;
    }

    /// Transition to stopping state
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.status.can_stop() {
            return Err(format!(
                "Cannot stop service in {} state",
                self.status.display_string()
            ));
        }
        self.status = ServiceStatus::Stopping;
        Ok(())
    }

    /// Transition to stopped state
    pub fn mark_stopped(&mut self) {
        self.status = ServiceStatus::Stopped;
    }

    /// Transition to failed state
    pub fn mark_failed(&mut self, error: String) {
        self.status = ServiceStatus::Failed(error);
    }

    /// Check if the service can be started
    pub fn can_start(&self) -> bool {
        self.status.can_start()
    }

    /// Check if the service can be stopped
    pub fn can_stop(&self) -> bool {
        self.status.can_stop()
    }
}

/// Status of a service
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed(String),
}

impl ServiceStatus {
    /// Check if the service can be started
    pub fn can_start(&self) -> bool {
        matches!(self, ServiceStatus::Stopped | ServiceStatus::Failed(_))
    }

    /// Check if the service can be stopped
    pub fn can_stop(&self) -> bool {
        matches!(self, ServiceStatus::Running)
    }

    /// Check if the service is in a transitional state
    pub fn is_transitioning(&self) -> bool {
        matches!(self, ServiceStatus::Starting | ServiceStatus::Stopping)
    }

    /// Check if the service is running
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceStatus::Running)
    }

    /// Check if the service has failed
    pub fn is_failed(&self) -> bool {
        matches!(self, ServiceStatus::Failed(_))
    }

    /// Get a display string for the status
    pub fn display_string(&self) -> &str {
        match self {
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::Starting => "Starting...",
            ServiceStatus::Running => "Running",
            ServiceStatus::Stopping => "Stopping...",
            ServiceStatus::Failed(_) => "Failed",
        }
    }

    /// Get the error message if failed
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ServiceStatus::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Information about an endpoint
#[derive(Debug, Clone)]
pub struct EndpointInfo {
    pub method: String,
    pub path: String,
}

/// Recording session state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecordingSession {
    pub id: String,
    pub target_url: String,
    pub proxy_port: u16,
    pub is_active: bool,
    pub captured_requests: Vec<RecordedRequest>,
}

/// Editor state
#[derive(Debug, Clone, Default)]
pub struct EditorState {
    pub content: String,
    pub dirty: bool,
    pub selected_service: Option<String>,
    pub loading: bool,
    pub saving: bool,
}

/// GUI configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuiConfig {
    pub services_directory: PathBuf,
    pub default_port: u16,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            services_directory: PathBuf::from("services"),
            default_port: 8080,
        }
    }
}

/// Stored details about generated code artifacts
#[derive(Debug, Clone, Default)]
pub struct GuiCodegenState {
    pub last_target: Option<String>,
    pub last_output: Option<String>,
}

/// Captured HTTP request data for recording sessions
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecordedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}
