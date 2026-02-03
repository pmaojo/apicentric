//! The cloud API.
//!
//! This module provides a cloud API for managing services.

pub mod api;
pub mod api_types;
pub mod cors;
pub mod error;
pub mod handlers;
pub mod iot_handlers;
pub mod monitoring;
pub mod recording_session;
pub mod server;
pub mod websocket;

pub use api_types::*;
pub use cors::create_cors_layer;
pub use error::{ApiError, ApiErrorCode, ErrorResponse};
pub use monitoring::{Metrics, MetricsCollector, StructuredLog};
pub use server::CloudServer;
pub use websocket::{broadcast_service_status, ws_handler, ServiceStatusUpdate, WebSocketState};
