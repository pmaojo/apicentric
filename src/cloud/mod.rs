//! The cloud API.
//!
//! This module provides a cloud API for managing services.

pub mod api;
pub mod cors;
pub mod error;
pub mod handlers;
pub mod iot_handlers;
pub mod monitoring;
pub mod recording_session;
pub mod server;
pub mod websocket;

// New modules
pub mod ai_handlers;
pub mod codegen_handlers;
pub mod fs_utils;
pub mod marketplace_handlers;
pub mod metrics_handlers;
pub mod recording_handlers;
pub mod types;

pub use cors::create_cors_layer;
pub use error::{ApiError, ApiErrorCode, ErrorResponse};
pub use monitoring::{Metrics, MetricsCollector, StructuredLog};
pub use server::CloudServer;
pub use websocket::{broadcast_service_status, ws_handler, ServiceStatusUpdate, WebSocketState};

// Re-export common types
pub use types::ApiResponse;
