//! Request Router - Routes incoming requests to appropriate service instances

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use futures_util::SinkExt;
use http_body_util::{Full, StreamBody};
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::simulator::config::EndpointDefinition;
use crate::simulator::template::{TemplateContext, TemplateEngine};
use tracing::info;

/// Request router that maps requests to service instances
pub struct RequestRouter {
    /// Maps base paths to service names
    service_mappings: HashMap<String, String>,
}

impl RequestRouter {
    /// Create a new request router
    pub fn new() -> Self {
        Self {
            service_mappings: HashMap::new(),
        }
    }

    /// Route a request to the appropriate service
    /// Returns the service name if a match is found
    pub fn route_request(&self, request_path: &str) -> Option<String> {
        // Find the longest matching base path
        let mut best_match: Option<(&String, &String)> = None;

        for (base_path, service_name) in &self.service_mappings {
            if request_path.starts_with(base_path) {
                match best_match {
                    None => best_match = Some((base_path, service_name)),
                    Some((current_best_path, _)) => {
                        if base_path.len() > current_best_path.len() {
                            best_match = Some((base_path, service_name));
                        }
                    }
                }
            }
        }

        best_match.map(|(_, service_name)| service_name.clone())
    }

    /// Register service routes for a service
    pub fn register_service_routes(&mut self, service_name: &str, base_path: &str) {
        // Normalize base path (ensure it starts with / and doesn't end with /)
        let normalized_path = self.normalize_path(base_path);

        self.service_mappings
            .insert(normalized_path, service_name.to_string());

        info!(
            target: "simulator",
            service = %service_name,
            path = %base_path,
            "Routes registered for service"
        );
    }

    /// Unregister service routes for a service
    pub fn unregister_service_routes(&mut self, service_name: &str) {
        // Find and remove all mappings for this service
        let paths_to_remove: Vec<String> = self
            .service_mappings
            .iter()
            .filter(|(_, name)| *name == service_name)
            .map(|(path, _)| path.clone())
            .collect();

        for path in paths_to_remove {
            self.service_mappings.remove(&path);
        }

        info!(
            target: "simulator",
            service = %service_name,
            "Routes unregistered for service"
        );
    }

    /// Clear all route mappings
    pub fn clear_all_routes(&mut self) {
        self.service_mappings.clear();
        info!(target: "simulator", "All route mappings cleared");
    }

    /// Get all registered routes
    pub fn get_all_routes(&self) -> &HashMap<String, String> {
        &self.service_mappings
    }

    /// Get routes for a specific service
    pub fn get_service_routes(&self, service_name: &str) -> Vec<String> {
        self.service_mappings
            .iter()
            .filter(|(_, name)| *name == service_name)
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Check if a service has registered routes
    pub fn has_service_routes(&self, service_name: &str) -> bool {
        self.service_mappings
            .values()
            .any(|name| name == service_name)
    }

    /// Get the number of registered route mappings
    pub fn routes_count(&self) -> usize {
        self.service_mappings.len()
    }

    /// Normalize a path for consistent routing
    fn normalize_path(&self, path: &str) -> String {
        let mut normalized = path.to_string();

        // Ensure path starts with /
        if !normalized.starts_with('/') {
            normalized = format!("/{}", normalized);
        }

        // Remove trailing / unless it's the root path
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }

        normalized
    }

    /// Extract the relative path after removing the base path
    pub fn extract_relative_path(&self, request_path: &str, base_path: &str) -> String {
        let normalized_base = self.normalize_path(base_path);

        if request_path.starts_with(&normalized_base) {
            let relative = &request_path[normalized_base.len()..];
            if relative.is_empty() {
                "/".to_string()
            } else if relative.starts_with('/') {
                relative.to_string()
            } else {
                format!("/{}", relative)
            }
        } else {
            request_path.to_string()
        }
    }

    /// Get routing statistics
    pub fn get_routing_stats(&self) -> RoutingStats {
        let mut service_counts = HashMap::new();

        for service_name in self.service_mappings.values() {
            *service_counts.entry(service_name.clone()).or_insert(0) += 1;
        }

        RoutingStats {
            total_routes: self.service_mappings.len(),
            services_count: service_counts.len(),
            service_route_counts: service_counts,
        }
    }
}

/// Handle a WebSocket upgrade and send templated messages
pub async fn handle_websocket_connection(
    req: Request<hyper::body::Incoming>,
    endpoint: &EndpointDefinition,
    engine: Arc<TemplateEngine>,
    context: TemplateContext,
) -> Response<Full<Bytes>> {
    let (parts, body) = req.into_parts();
    let req_head = Request::from_parts(parts.clone(), ());
    // Create handshake response
    let response = tokio_tungstenite::tungstenite::handshake::server::create_response(&req_head)
        .map(|resp| {
            let (parts_resp, _) = resp.into_parts();
            Response::from_parts(parts_resp, Full::new(Bytes::new()))
        })
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::new()))
                .unwrap()
        });

    let upgrade = hyper::upgrade::on(Request::from_parts(parts, body));
    let endpoint_clone = endpoint.clone();
    tokio::spawn(async move {
        if let Ok(upgraded) = upgrade.await {
            if let Ok(mut ws) = accept_async(TokioIo::new(upgraded)).await {
                if let Some(cfg) = endpoint_clone.stream.as_ref() {
                    for tpl in &cfg.initial {
                        if let Ok(msg) = engine.render(tpl, &context) {
                            let _ = ws.send(Message::Text(msg)).await;
                        }
                    }
                    if let Some(periodic) = &cfg.periodic {
                        let mut ticker = interval(Duration::from_millis(periodic.interval_ms));
                        loop {
                            ticker.tick().await;
                            if let Ok(msg) = engine.render(&periodic.message, &context) {
                                if ws.send(Message::Text(msg)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
                let _ = ws.close(None).await;
            }
        }
    });

    response
}

/// Create a Server-Sent Events response with optional periodic messages
pub fn handle_sse_connection(
    endpoint: &EndpointDefinition,
    engine: Arc<TemplateEngine>,
    context: TemplateContext,
) -> Response<StreamBody<UnboundedReceiverStream<Result<Bytes, Infallible>>>> {
    let (tx, rx) = mpsc::unbounded_channel();

    if let Some(cfg) = endpoint.stream.as_ref() {
        for tpl in &cfg.initial {
            if let Ok(msg) = engine.render(tpl, &context) {
                let _ = tx.send(Ok(Bytes::from(format!("data: {}\n\n", msg))));
            }
        }

        if let Some(periodic) = &cfg.periodic {
            let mut ticker = interval(Duration::from_millis(periodic.interval_ms));
            let tx_clone = tx.clone();
            let msg_tpl = periodic.message.clone();
            let engine_clone = engine.clone();
            let ctx_clone = context.clone();
            tokio::spawn(async move {
                loop {
                    ticker.tick().await;
                    if let Ok(msg) = engine_clone.render(&msg_tpl, &ctx_clone) {
                        if tx_clone
                            .send(Ok(Bytes::from(format!("data: {}\n\n", msg))))
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            });
        }
    }

    let stream = UnboundedReceiverStream::new(rx);
    let body = StreamBody::new(stream);
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(body)
        .unwrap()
}

/// Statistics about the current routing configuration
#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_routes: usize,
    pub services_count: usize,
    pub service_route_counts: HashMap<String, usize>,
}

impl Default for RequestRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_registration() {
        let mut router = RequestRouter::new();

        router.register_service_routes("user-service", "/api/v1/users");
        router.register_service_routes("order-service", "/api/v1/orders");

        assert_eq!(router.routes_count(), 2);
        assert!(router.has_service_routes("user-service"));
        assert!(router.has_service_routes("order-service"));
        assert!(!router.has_service_routes("nonexistent-service"));
    }

    #[test]
    fn test_request_routing() {
        let mut router = RequestRouter::new();

        router.register_service_routes("user-service", "/api/v1/users");
        router.register_service_routes("order-service", "/api/v1/orders");

        // Test exact matches
        assert_eq!(
            router.route_request("/api/v1/users"),
            Some("user-service".to_string())
        );
        assert_eq!(
            router.route_request("/api/v1/orders"),
            Some("order-service".to_string())
        );

        // Test sub-path matches
        assert_eq!(
            router.route_request("/api/v1/users/123"),
            Some("user-service".to_string())
        );
        assert_eq!(
            router.route_request("/api/v1/orders/456/items"),
            Some("order-service".to_string())
        );

        // Test no match
        assert_eq!(router.route_request("/api/v2/products"), None);
        assert_eq!(router.route_request("/health"), None);
    }

    #[test]
    fn test_longest_path_matching() {
        let mut router = RequestRouter::new();

        // Register overlapping paths
        router.register_service_routes("api-service", "/api");
        router.register_service_routes("user-service", "/api/users");
        router.register_service_routes("admin-service", "/api/users/admin");

        // Should match the longest path
        assert_eq!(
            router.route_request("/api/users/admin/settings"),
            Some("admin-service".to_string())
        );
        assert_eq!(
            router.route_request("/api/users/123"),
            Some("user-service".to_string())
        );
        assert_eq!(
            router.route_request("/api/health"),
            Some("api-service".to_string())
        );
    }

    #[test]
    fn test_path_normalization() {
        let router = RequestRouter::new();

        assert_eq!(router.normalize_path("/api/v1"), "/api/v1");
        assert_eq!(router.normalize_path("api/v1"), "/api/v1");
        assert_eq!(router.normalize_path("/api/v1/"), "/api/v1");
        assert_eq!(router.normalize_path("api/v1/"), "/api/v1");
        assert_eq!(router.normalize_path("/"), "/");
        assert_eq!(router.normalize_path(""), "/");
    }

    #[test]
    fn test_relative_path_extraction() {
        let router = RequestRouter::new();

        assert_eq!(
            router.extract_relative_path("/api/v1/users/123", "/api/v1/users"),
            "/123"
        );
        assert_eq!(
            router.extract_relative_path("/api/v1/users", "/api/v1/users"),
            "/"
        );
        assert_eq!(
            router.extract_relative_path("/api/v1/users/123/orders", "/api/v1/users"),
            "/123/orders"
        );

        // Test with non-matching base path
        assert_eq!(
            router.extract_relative_path("/different/path", "/api/v1/users"),
            "/different/path"
        );
    }

    #[test]
    fn test_service_route_unregistration() {
        let mut router = RequestRouter::new();

        router.register_service_routes("user-service", "/api/v1/users");
        router.register_service_routes("user-service", "/api/v2/users");
        router.register_service_routes("order-service", "/api/v1/orders");

        assert_eq!(router.routes_count(), 3);

        router.unregister_service_routes("user-service");

        assert_eq!(router.routes_count(), 1);
        assert!(!router.has_service_routes("user-service"));
        assert!(router.has_service_routes("order-service"));
    }

    #[test]
    fn test_clear_all_routes() {
        let mut router = RequestRouter::new();

        router.register_service_routes("service1", "/api/v1");
        router.register_service_routes("service2", "/api/v2");

        assert_eq!(router.routes_count(), 2);

        router.clear_all_routes();

        assert_eq!(router.routes_count(), 0);
        assert!(!router.has_service_routes("service1"));
        assert!(!router.has_service_routes("service2"));
    }

    #[test]
    fn test_get_service_routes() {
        let mut router = RequestRouter::new();

        router.register_service_routes("user-service", "/api/v1/users");
        router.register_service_routes("user-service", "/api/v2/users");
        router.register_service_routes("order-service", "/api/v1/orders");

        let user_routes = router.get_service_routes("user-service");
        assert_eq!(user_routes.len(), 2);
        assert!(user_routes.contains(&"/api/v1/users".to_string()));
        assert!(user_routes.contains(&"/api/v2/users".to_string()));

        let order_routes = router.get_service_routes("order-service");
        assert_eq!(order_routes.len(), 1);
        assert!(order_routes.contains(&"/api/v1/orders".to_string()));
    }

    #[test]
    fn test_routing_stats() {
        let mut router = RequestRouter::new();

        router.register_service_routes("user-service", "/api/v1/users");
        router.register_service_routes("user-service", "/api/v2/users");
        router.register_service_routes("order-service", "/api/v1/orders");

        let stats = router.get_routing_stats();

        assert_eq!(stats.total_routes, 3);
        assert_eq!(stats.services_count, 2);
        assert_eq!(stats.service_route_counts.get("user-service"), Some(&2));
        assert_eq!(stats.service_route_counts.get("order-service"), Some(&1));
    }
}
