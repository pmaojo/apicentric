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
    _ServiceStatusChanged(String, super::models::ServiceStatus),

    // AI Generation
    AiGenerate(String),
    _AiGenerationComplete(Result<String, String>),
    _AiApplyYaml(String),

    // Recording Mode
    _StartRecording(String),
    _StopRecording,
    _CaptureRequest(CapturedRequest),
    _GenerateFromRecording,

    // Editor
    _LoadServiceInEditor(String),
    _SaveEditorContent,
    _EditorContentChanged(String),

    // Logs
    _NewRequestLog(super::models::RequestLogEntry),
    _ClearLogs,
    _FilterLogsBy(super::models::LogFilter),

    // Import/Export
    _ImportFile(PathBuf),
    _ExportService(String, ExportFormat),
    _BatchImport(Vec<PathBuf>),

    // Code Generation
    _GenerateCode(String, CodeGenTarget),
    _CopyToClipboard(String),
    _SaveGeneratedCode(PathBuf, String),

    // Configuration
    _UpdateConfig(super::models::GuiConfig),
    _SaveConfig,
    _LoadConfig,
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
    _Yaml,
    _Json,
    _Postman,
}

/// Code generation target options
#[derive(Debug, Clone)]
pub enum CodeGenTarget {
    _TypeScript,
    _JavaScript,
    _Python,
    _Go,
    _Rust,
}
