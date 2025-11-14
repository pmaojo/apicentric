//! The cloud API.
//!
//! This module provides a cloud API for managing services.

pub mod server;
pub mod api;
pub mod handlers;
pub mod recording_session;
pub mod error;
pub mod websocket;
pub mod cors;
pub mod monitoring;

pub use server::CloudServer;
pub use error::{ApiError, ApiErrorCode, ErrorResponse};
pub use websocket::{WebSocketState, ws_handler, broadcast_service_status, ServiceStatusUpdate};
pub use cors::create_cors_layer;
pub use monitoring::{MetricsCollector, Metrics, StructuredLog};
