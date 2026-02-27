//! Request Router - Routes incoming requests to appropriate service instances

pub mod core;
pub mod sse;
#[cfg(feature = "websockets")]
pub mod websocket;

pub use core::{RequestRouter, RoutingStats};
pub use sse::handle_sse_connection;
#[cfg(feature = "websockets")]
pub use websocket::handle_websocket_connection;

#[cfg(not(feature = "websockets"))]
pub async fn handle_websocket_connection(
    _req: hyper::Request<hyper::body::Incoming>,
    _endpoint: &crate::simulator::config::EndpointDefinition,
    _engine: std::sync::Arc<crate::simulator::template::TemplateEngine>,
    _context: crate::simulator::template::TemplateContext,
) -> hyper::Response<http_body_util::Full<bytes::Bytes>> {
    hyper::Response::builder()
        .status(hyper::StatusCode::NOT_IMPLEMENTED)
        .body(http_body_util::Full::new(bytes::Bytes::from(
            "WebSockets not enabled",
        )))
        .unwrap()
}
