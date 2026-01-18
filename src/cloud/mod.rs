//! The cloud API.
//!
//! This module provides a cloud API for managing services.

<<<<<<< HEAD
pub mod api;
pub mod cors;
pub mod error;
pub mod handlers;
#[cfg(feature = "iot")]
pub mod iot_handlers;
pub mod monitoring;
pub mod recording_session;
pub mod server;
pub mod websocket;

pub use cors::create_cors_layer;
pub use error::{ApiError, ApiErrorCode, ErrorResponse};
pub use monitoring::{Metrics, MetricsCollector, StructuredLog};
pub use server::CloudServer;
pub use websocket::{broadcast_service_status, ws_handler, ServiceStatusUpdate, WebSocketState};
=======
pub mod server;
pub mod api;
pub mod handlers;

pub use server::CloudServer;
>>>>>>> origin/main
