//! The cloud API.
//!
//! This module provides a cloud API for managing services.

pub mod api;
pub mod cors;
pub mod error;
pub mod handlers;
pub mod monitoring;
pub mod recording_session;
pub mod server;
pub mod websocket;

pub use cors::create_cors_layer;
pub use error::{ApiError, ApiErrorCode, ErrorResponse};
pub use monitoring::{Metrics, MetricsCollector, StructuredLog};
pub use server::CloudServer;
pub use websocket::{broadcast_service_status, ws_handler, ServiceStatusUpdate, WebSocketState};
