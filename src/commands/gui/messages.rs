//! GUI Message Types
//!
//! This module defines the message types used for communication
//! between the GUI components and the event handler.

use std::path::PathBuf;
use super::models::ServiceInfo;

/// Messages sent from the GUI to the event handler (User Actions)
#[derive(Debug, Clone)]
pub enum GuiMessage {
    // Service Management
    StartService(String),
    StopService(String),
    RefreshServices,
    ServiceStatusChanged(String, super::models::ServiceStatus),

    // AI Generation
    AiGenerate(String),
    AiGenerationComplete(Result<String, String>),
    AiApplyYaml(String),

    // Recording Mode
    StartRecording(String),
    StopRecording,
    CaptureRequest(CapturedRequest),
    GenerateFromRecording,

    // Editor
    LoadServiceInEditor(String),
    SaveEditorContent,
    EditorContentChanged(String),

    // Logs
    NewRequestLog(super::models::RequestLogEntry),
    ClearLogs,
    FilterLogsBy(super::models::LogFilter),

    // Import/Export
    ImportFile(PathBuf),
    ExportService(String, ExportFormat),
    BatchImport(Vec<PathBuf>),

    // Code Generation
    GenerateCode(String, CodeGenTarget),
    CopyToClipboard(String),
    SaveGeneratedCode(PathBuf, String),

    // Configuration
    UpdateConfig(super::models::GuiConfig),
    SaveConfig,
    LoadConfig,
}

/// Events sent from background tasks to the GUI (System Events)
#[derive(Debug)]
pub enum GuiSystemEvent {
    SimulatorStarted,
    SimulatorStopped,
    ServicesLoaded(Vec<ServiceInfo>),
    ServicesRefreshed(Vec<ServiceInfo>),
    Error(String),
    Log(String),
}

/// Information about a captured HTTP request
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Yaml,
    Json,
    Postman,
}

/// Code generation target options
#[derive(Debug, Clone)]
pub enum CodeGenTarget {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Rust,
}
