//! Service Instance - Individual service implementation with state management

pub mod routing;
pub mod graphql;
pub mod state;
pub mod scenario;
pub mod state_service;
pub mod router;
pub mod http_server;

pub use graphql::*;
pub use routing::*;
pub use state::*;
pub use scenario::ScenarioService;
pub use state_service::StateService;
pub use router::{RequestRouter, DefaultRouter};
pub use http_server::HttpServer;

use crate::errors::{PulseError, PulseResult};
use crate::simulator::config::{
    EndpointDefinition, ResponseDefinition, ScenarioDefinition, ScenarioStrategy, ServiceDefinition,
};
use crate::simulator::log::RequestLogEntry;
use crate::simulator::template::{RequestContext, TemplateContext, TemplateEngine};
use crate::storage::Storage;
use bytes::Bytes;
use http_body_util::Full;
use hyper::header::HOST;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use deno_core::{JsRuntime, RuntimeOptions};

/// Individual service instance with HTTP server capabilities
pub struct ServiceInstance {
    definition: ServiceDefinition,
    port: u16,
    state: Arc<RwLock<ServiceState>>,
    template_engine: Arc<TemplateEngine>,
    server_handle: Option<JoinHandle<()>>,
    is_running: bool,
    active_scenario: Arc<RwLock<Option<String>>>,
    graphql: Option<Arc<GraphQLMocks>>,
}

impl ServiceInstance {
    /// Create a new service instance
    pub fn new(
        definition: ServiceDefinition,
        port: u16,
        storage: Arc<dyn Storage>,
        log_sender: broadcast::Sender<RequestLogEntry>,
    ) -> PulseResult<Self> {
        // Initialize state with fixtures and bucket from definition
        let state = ServiceState::new(
            definition.fixtures.clone(),
            definition.bucket.clone(),
            storage.clone(),
            Some(log_sender),
        );

        // persist service definition
        let _ = storage.save_service(&definition);

        // Initialize template engine and register bucket helpers
        let mut template_engine = TemplateEngine::new()?;
        template_engine.register_bucket_helpers(state.bucket())?;

        let graphql = if let Some(ref gql_cfg) = definition.graphql {
            Some(Arc::new(load_graphql_mocks(gql_cfg)?))
        } else {
            None
        };

        Ok(Self {
            definition,
            port,
            state: Arc::new(RwLock::new(state)),
            template_engine: Arc::new(template_engine),
            server_handle: None,
            is_running: false,
            active_scenario: Arc::new(RwLock::new(None)),
            graphql,
        })
    }

    /// Start the service HTTP server
    pub async fn start(&mut self) -> PulseResult<()> {
        if self.is_running {
            return Err(PulseError::runtime_error(
                format!("Service '{}' is already running", self.definition.name),
                None::<String>,
            ));
        }

        // Create TCP listener for the service
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to bind to port {}: {}", self.port, e),
                Some("Port may already be in use or unavailable"),
            )
        })?;

        // Clone necessary data for the server task
        let service_name = self.definition.name.clone();
        let base_path = self.definition.server.base_path.clone();
        let endpoints = self.definition.endpoints.clone();
        let state = Arc::clone(&self.state);
        let template_engine = Arc::clone(&self.template_engine);
        let cors = self.definition.server.cors.clone();
        let proxy = self.definition.server.proxy_base_url.clone();
        let active_scenario = Arc::clone(&self.active_scenario);
        let graphql = self.graphql.clone();

        // Spawn the HTTP server task
        let server_handle = tokio::spawn(async move {
            println!(
                "🚀 Started service '{}' on port {} at {}",
                service_name,
                addr.port(),
                base_path
            );

            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let io = TokioIo::new(stream);
                        let service_name = service_name.clone();
                        let base_path = base_path.clone();
                        let endpoints = endpoints.clone();
                        let state = Arc::clone(&state);
                        let template_engine = Arc::clone(&template_engine);
                        let cors_cfg = cors.clone();
                        let proxy_cfg = proxy.clone();
                        let scenario_cfg_outer = Arc::clone(&active_scenario);
                        let graphql_cfg_outer = graphql.clone();

                        // Handle each connection
                        let connection_service_name = service_name.clone();
                        tokio::task::spawn(async move {
                            let service = service_fn(move |req| {
                                let service_name = service_name.clone();
                                let base_path = base_path.clone();
                                let endpoints = endpoints.clone();
                                let state = Arc::clone(&state);
                                let template_engine = Arc::clone(&template_engine);
                                let cors_cfg = cors_cfg.clone();
                                let proxy_cfg = proxy_cfg.clone();
                                let scenario_cfg = Arc::clone(&scenario_cfg_outer);
                                let graphql_cfg = graphql_cfg_outer.clone();

                                async move {
                                    match Self::handle_request_static(
                                        req,
                                        service_name.clone(),
                                        base_path,
                                        endpoints,
                                        state,
                                        template_engine,
                                        cors_cfg,
                                        proxy_cfg,
                                        scenario_cfg,
                                        graphql_cfg,
                                    )
                                    .await
                                    {
                                        Ok(resp) => Ok::<_, Infallible>(resp),
                                        Err(err) => {
                                            eprintln!(
                                                "Error handling request for service '{}': {}",
                                                service_name, err
                                            );
                                            let fallback = match Response::builder()
                                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                .header("content-type", "application/json")
                                                .body(Full::new(Bytes::from(format!(
                                                    r#"{{"error": "{}"}}"#,
                                                    err
                                                ))))
                                            {
                                                Ok(r) => r,
                                                Err(_) => Response::new(Full::new(Bytes::new())),
                                            };
                                            Ok::<_, Infallible>(fallback)
                                        }
                                    }
                                }
                            });

                            if let Err(err) =
                                http1::Builder::new().serve_connection(io, service).await
                            {
                                eprintln!(
                                    "Error serving connection for service '{}': {:?}",
                                    connection_service_name, err
                                );
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to accept connection for service '{}': {}",
                            service_name, e
                        );
                        break;
                    }
                }
            }
        });

        self.server_handle = Some(server_handle);
        self.is_running = true;

        Ok(())
    }

    /// Stop the service HTTP server
    pub async fn stop(&mut self) -> PulseResult<()> {
        if !self.is_running {
            return Ok(()); // Already stopped
        }

        // Stop the server if it's running
        if let Some(handle) = self.server_handle.take() {
            handle.abort();

            // Wait a moment for graceful shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        self.is_running = false;

        println!("🛑 Stopped service '{}'", self.definition.name);

        Ok(())
    }

    /// Handle a single HTTP request (for external use)
    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> PulseResult<Response<Full<Bytes>>> {
        if !self.is_running {
            return Err(PulseError::runtime_error(
                format!("Service '{}' is not running", self.definition.name),
                Some("Start the service before handling requests"),
            ));
        }

        Self::handle_request_static(
            req,
            self.definition.name.clone(),
            self.definition.server.base_path.clone(),
            self.definition.endpoints.clone(),
            Arc::clone(&self.state),
            Arc::clone(&self.template_engine),
            self.definition.server.cors.clone(),
            self.definition.server.proxy_base_url.clone(),
            Arc::clone(&self.active_scenario),
            self.graphql.clone(),
        )
        .await
    }

    /// Check if the service is currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the service port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the service base path
    pub fn base_path(&self) -> &str {
        &self.definition.server.base_path
    }

    /// Get the service name
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// Get the number of endpoints defined for this service
    pub fn endpoints_count(&self) -> usize {
        self.definition.endpoints.len()
    }

    /// Get all endpoint definitions
    pub fn endpoints(&self) -> &[EndpointDefinition] {
        &self.definition.endpoints
    }

    /// Get the service definition
    pub fn definition(&self) -> &ServiceDefinition {
        &self.definition
    }

    /// Set the active scenario for this service
    pub async fn set_scenario(&self, scenario: Option<String>) {
        let mut guard = self.active_scenario.write().await;
        *guard = scenario;
    }

    /// Get the currently active scenario
    pub async fn get_scenario(&self) -> Option<String> {
        self.active_scenario.read().await.clone()
    }

    /// Update service state
    pub async fn update_state(&self, key: &str, value: Value) {
        let mut state = self.state.write().await;
        state.set_runtime_data(key.to_string(), value);
    }

    /// Get value from service state
    pub async fn get_state(&self, key: &str) -> Option<Value> {
        let state = self.state.read().await;
        state
            .get_runtime_data(key)
            .cloned()
            .or_else(|| state.get_fixture(key).cloned())
    }

    /// Get recent request logs
    pub async fn get_logs(&self, limit: usize) -> Vec<RequestLogEntry> {
        let state = self.state.read().await;
        state.get_logs(limit)
    }

    /// Query request logs with optional filters
    pub async fn query_logs(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> Vec<RequestLogEntry> {
        let state = self.state.read().await;
        state.query_logs(service, route, method, status, limit)
    }

    /// Internal helper to record a request log entry
    async fn record_log(
        state: &Arc<RwLock<ServiceState>>,
        service: &str,
        endpoint: Option<usize>,
        method: &str,
        path: &str,
        status: u16,
    ) {
        let mut guard = state.write().await;
        guard.add_log_entry(RequestLogEntry::new(
            service.to_string(),
            endpoint,
            method.to_string(),
            path.to_string(),
            status,
        ));
    }

    /// Get all fixtures
    pub async fn get_fixtures(&self) -> HashMap<String, Value> {
        let state = self.state.read().await;
        state.all_fixtures().clone()
    }

    /// Update a fixture
    pub async fn update_fixture(&self, key: &str, value: Value) {
        let mut state = self.state.write().await;
        state.set_fixture(key.to_string(), value);
    }

    /// Remove a fixture
    pub async fn remove_fixture(&self, key: &str) -> Option<Value> {
        let mut state = self.state.write().await;
        state.remove_fixture(key)
    }

    /// Add an item to a fixture array
    pub async fn add_to_fixture_array(&self, fixture_key: &str, item: Value) -> PulseResult<()> {
        let mut state = self.state.write().await;
        state.add_to_fixture_array(fixture_key, item)
    }

    /// Remove an item from a fixture array by index
    pub async fn remove_from_fixture_array(
        &self,
        fixture_key: &str,
        index: usize,
    ) -> PulseResult<Value> {
        let mut state = self.state.write().await;
        state.remove_from_fixture_array(fixture_key, index)
    }

    /// Update an item in a fixture array by index
    pub async fn update_fixture_array_item(
        &self,
        fixture_key: &str,
        index: usize,
        item: Value,
    ) -> PulseResult<()> {
        let mut state = self.state.write().await;
        state.update_fixture_array_item(fixture_key, index, item)
    }

    /// Find and update an item in a fixture array by field value
    pub async fn update_fixture_array_item_by_field(
        &self,
        fixture_key: &str,
        field: &str,
        field_value: &Value,
        new_item: Value,
    ) -> PulseResult<bool> {
        let mut state = self.state.write().await;
        state.update_fixture_array_item_by_field(fixture_key, field, field_value, new_item)
    }

    /// Find and remove an item from a fixture array by field value
    pub async fn remove_fixture_array_item_by_field(
        &self,
        fixture_key: &str,
        field: &str,
        field_value: &Value,
    ) -> PulseResult<Option<Value>> {
        let mut state = self.state.write().await;
        state.remove_fixture_array_item_by_field(fixture_key, field, field_value)
    }

    /// Reset fixtures to their initial state
    pub async fn reset_fixtures(&self) {
        let mut state = self.state.write().await;
        state.reset_fixtures();
    }

    /// Get runtime data
    pub async fn get_runtime_data(&self, key: &str) -> Option<Value> {
        let state = self.state.read().await;
        state.get_runtime_data(key).cloned()
    }

    /// Set runtime data
    pub async fn set_runtime_data(&self, key: &str, value: Value) {
        let mut state = self.state.write().await;
        state.set_runtime_data(key.to_string(), value);
    }

    /// Remove runtime data
    pub async fn remove_runtime_data(&self, key: &str) -> Option<Value> {
        let mut state = self.state.write().await;
        state.remove_runtime_data(key)
    }

    /// Clear all runtime data
    pub async fn clear_runtime_data(&self) {
        let mut state = self.state.write().await;
        state.clear_runtime_data();
    }

    /// Check if a fixture exists
    pub async fn has_fixture(&self, key: &str) -> bool {
        let state = self.state.read().await;
        state.has_fixture(key)
    }

    /// Check if runtime data exists
    pub async fn has_runtime_data(&self, key: &str) -> bool {
        let state = self.state.read().await;
        state.has_runtime_data(key)
    }

    /// Get fixture and runtime data counts
    pub async fn get_state_info(&self) -> (usize, usize) {
        let state = self.state.read().await;
        (state.fixture_count(), state.runtime_data_count())
    }

    /// Get service information for status reporting
    pub fn get_info(&self) -> crate::simulator::ServiceInfo {
        crate::simulator::ServiceInfo {
            name: self.definition.name.clone(),
            port: self.port,
            base_path: self.definition.server.base_path.clone(),
            endpoints_count: self.definition.endpoints.len(),
            is_running: self.is_running,
        }
    }

    /// Find an endpoint by method, path and headers with parameter extraction
    pub fn find_endpoint_with_params(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Option<RouteMatch> {
        for (index, endpoint) in self.definition.endpoints.iter().enumerate() {
            if endpoint.method.to_uppercase() == method.to_uppercase()
                && Self::headers_match(endpoint, headers)
            {
                if let Some(path_params) = self.extract_path_parameters(&endpoint.path, path) {
                    return Some(RouteMatch {
                        endpoint: endpoint.clone(),
                        endpoint_index: index,
                        path_params,
                    });
                }
            }
        }
        None
    }

    /// Find an endpoint by method, path and headers (legacy reference)
    pub fn find_endpoint(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Option<&EndpointDefinition> {
        // Use the original logic for backward compatibility
        for endpoint in &self.definition.endpoints {
            if endpoint.method.to_uppercase() == method.to_uppercase()
                && Self::headers_match(endpoint, headers)
            {
                if self.extract_path_parameters(&endpoint.path, path).is_some() {
                    return Some(endpoint);
                }
            }
        }
        None
    }

    /// Check if request headers satisfy an endpoint's header_match criteria
    fn headers_match(endpoint: &EndpointDefinition, headers: &HashMap<String, String>) -> bool {
        if let Some(required) = &endpoint.header_match {
            for (k, v) in required {
                match headers.get(&k.to_lowercase()) {
                    Some(val) if val.eq_ignore_ascii_case(v) => {}
                    _ => return false,
                }
            }
        }
        true
    }

    /// Extract path parameters from a request path against an endpoint path pattern
    fn extract_path_parameters(
        &self,
        endpoint_path: &str,
        request_path: &str,
    ) -> Option<PathParameters> {
        // Convert endpoint path pattern to regex
        let regex_pattern = self.endpoint_path_to_regex(endpoint_path);

        match Regex::new(&regex_pattern) {
            Ok(regex) => {
                if let Some(captures) = regex.captures(request_path) {
                    let mut params = PathParameters::new();

                    // Extract named parameters
                    for name in regex.capture_names().flatten() {
                        if let Some(matched) = captures.name(name) {
                            params.insert(name.to_string(), matched.as_str().to_string());
                        }
                    }

                    Some(params)
                } else {
                    None
                }
            }
            Err(_) => {
                // Fallback to exact matching if regex compilation fails
                if endpoint_path == request_path {
                    Some(PathParameters::new())
                } else {
                    None
                }
            }
        }
    }

    /// Convert an endpoint path pattern to a regex pattern
    fn endpoint_path_to_regex(&self, endpoint_path: &str) -> String {
        Self::endpoint_path_to_regex_static(endpoint_path)
    }

    /// Static request handler for use in the HTTP server
    async fn handle_request_static(
        req: Request<hyper::body::Incoming>,
        service_name: String,
        base_path: String,
        endpoints: Vec<EndpointDefinition>,
        state: Arc<RwLock<ServiceState>>,
        template_engine: Arc<TemplateEngine>,
        cors_cfg: Option<crate::simulator::config::CorsConfig>,
        proxy_base_url: Option<String>,
        active_scenario: Arc<RwLock<Option<String>>>,
        graphql: Option<Arc<GraphQLMocks>>,
    ) -> PulseResult<Response<Full<Bytes>>> {
        let (parts, body) = req.into_parts();
        let method = parts.method.as_str();
        let path = parts.uri.path();

        // Log incoming request
        println!(
            "🌐 [{}] {} {} - Origin: {}",
            service_name,
            method,
            path,
            parts
                .headers
                .get("origin")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("none")
        );

        // Log CORS configuration
        if let Some(ref cors) = cors_cfg {
            println!(
                "🔧 [{}] CORS enabled - Origins: {:?}, Methods: {:?}",
                service_name, cors.origins, cors.methods
            );
        } else {
            println!("⚠️ [{}] CORS not configured", service_name);
        }

        // Parse query parameters
        let query_params = parts
            .uri
            .query()
            .map(|q| {
                q.split('&')
                    .filter_map(|param| {
                        let mut parts = param.split('=');
                        match (parts.next(), parts.next()) {
                            (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                            _ => None,
                        }
                    })
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        // Parse headers
        let headers: HashMap<String, String> = parts
            .headers
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();

        // Handle CORS preflight
        if method == "OPTIONS" {
            println!("✈️ [{}] Handling CORS preflight for {}", service_name, path);

            let origin = headers.get("origin").cloned().unwrap_or_default();
            println!("🔍 [{}] Request origin: '{}'", service_name, origin);

            let allow_origin = match &cors_cfg {
                Some(cfg) => {
                    println!("✅ [{}] CORS config found: {:?}", service_name, cfg);
                    if cfg.origins.iter().any(|o| o == "*") {
                        println!("🌍 [{}] Wildcard origin allowed", service_name);
                        "*".to_string()
                    } else if cfg.origins.iter().any(|o| o.eq_ignore_ascii_case(&origin)) {
                        println!(
                            "✅ [{}] Origin '{}' explicitly allowed",
                            service_name, origin
                        );
                        origin.clone()
                    } else {
                        println!(
                            "⚠️ [{}] Origin '{}' not in allowed list, defaulting to wildcard",
                            service_name, origin
                        );
                        "*".to_string()
                    }
                }
                None => {
                    println!(
                        "⚠️ [{}] No CORS config, defaulting to wildcard",
                        service_name
                    );
                    "*".to_string()
                }
            };
            let req_headers = headers
                .get("access-control-request-headers")
                .cloned()
                .unwrap_or_else(|| {
                    cors_cfg
                        .as_ref()
                        .and_then(|c| c.headers.clone())
                        .map(|v| v.join(", "))
                        .unwrap_or_else(|| "Content-Type, Authorization".to_string())
                });
            let allow_methods = cors_cfg
                .as_ref()
                .and_then(|c| c.methods.clone())
                .map(|v| v.join(", "))
                .unwrap_or_else(|| "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string());

            println!("📤 [{}] CORS preflight response:", service_name);
            println!("   🌍 Access-Control-Allow-Origin: {}", allow_origin);
            println!("   🛠️ Access-Control-Allow-Methods: {}", allow_methods);
            println!("   📋 Access-Control-Allow-Headers: {}", req_headers);

            let resp = Response::builder()
                .status(StatusCode::NO_CONTENT)
                .header("access-control-allow-origin", &allow_origin)
                .header("access-control-allow-methods", &allow_methods)
                .header("access-control-allow-headers", &req_headers)
                .header("access-control-max-age", "86400")
                .body(Full::new(Bytes::from_static(b"")))
                .map_err(|e| {
                    PulseError::runtime_error(
                        format!("Failed to build CORS preflight response: {}", e),
                        None::<String>,
                    )
                })?;

            println!(
                "✅ [{}] CORS preflight response sent with status 204",
                service_name
            );
            Self::record_log(
                &state,
                &service_name,
                None,
                method,
                path,
                StatusCode::NO_CONTENT.as_u16(),
            )
            .await;
            return Ok(resp);
        }

        // Parse request body if present
        let body_bytes = match http_body_util::BodyExt::collect(body).await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                let resp = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"error": "Failed to read request body"}"#,
                    )))
                    .map_err(|e| {
                        PulseError::runtime_error(
                            format!("Failed to build bad request response: {}", e),
                            None::<String>,
                        )
                    })?;
                Self::record_log(
                    &state,
                    &service_name,
                    None,
                    method,
                    path,
                    StatusCode::BAD_REQUEST.as_u16(),
                )
                .await;
                return Ok(resp);
            }
        };

        let request_body = if !body_bytes.is_empty() {
            let body_str = String::from_utf8_lossy(&body_bytes);
            println!("📦 [{}] Request body received: {}", service_name, body_str);

            // Determine content type
            let content_type = parts
                .headers
                .get(hyper::header::CONTENT_TYPE)
                .and_then(|hv| hv.to_str().ok())
                .unwrap_or("")
                .to_lowercase();

            if content_type.contains("application/x-www-form-urlencoded") {
                // Parse form-encoded body
                let mut map = serde_json::Map::new();
                for (k, v) in url::form_urlencoded::parse(body_str.as_bytes()) {
                    map.insert(k.to_string(), Value::String(v.into_owned()));
                }
                Some(Value::Object(map))
            } else {
                // Try to parse as JSON
                serde_json::from_str::<Value>(&body_str).ok()
            }
        } else {
            None
        };

        // Remove base path from request path if it matches
        let relative_path = if path.starts_with(&base_path) {
            &path[base_path.len()..]
        } else {
            path
        };

        // Ensure relative path starts with '/'
        let relative_path = if relative_path.is_empty() || !relative_path.starts_with('/') {
            format!("/{}", relative_path.trim_start_matches('/'))
        } else {
            relative_path.to_string()
        };

        // Handle GraphQL endpoint if configured
        if let Some(gql) = &graphql {
            if let Some((resp, status)) = handle_graphql_request(
                gql,
                method,
                &relative_path,
                &body_bytes,
                &query_params,
                &headers,
                &template_engine,
                &state,
                &service_name,
                path,
            )
            .await
            {
                Self::record_log(&state, &service_name, None, method, path, status).await;
                return Ok(resp);
            }
        }

        // Internal logs endpoint
        if method == "GET" && relative_path == "/__mockforge/logs" {
            let limit = query_params
                .get("limit")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(100);
            let method_filter = query_params.get("method").map(|s| s.as_str());
            let route_filter = query_params.get("route").map(|s| s.as_str());
            let status_filter = query_params
                .get("status")
                .and_then(|v| v.parse::<u16>().ok());
            let logs = {
                let state = state.read().await;
                state.query_logs(
                    Some(&service_name),
                    route_filter,
                    method_filter,
                    status_filter,
                    limit,
                )
            };
            let body = serde_json::to_string(&logs)?;
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(body)))
                .map_err(|e| {
                    PulseError::runtime_error(
                        format!("Failed to build logs response: {}", e),
                        None::<String>,
                    )
                })?;
            Self::record_log(&state, &service_name, None, method, path, 200).await;
            return Ok(resp);
        }

        // Find matching endpoint with parameter extraction
        let route_match =
            Self::find_endpoint_with_params_static(&endpoints, method, &relative_path, &headers);

        match route_match {
            Some(route_match) => {
                // Evaluate conditions to find the right response
                let mut selected_response: Option<ResponseDefinition> = None;
                let mut selected_status = 200u16;

                // Try to match explicit or rotating scenarios
                let active = active_scenario.read().await.clone();
                if let Some((status, resp)) = Self::match_scenario(
                    &route_match.endpoint,
                    &state,
                    route_match.endpoint_index,
                    active,
                    &query_params,
                    &headers,
                    &request_body,
                )
                .await
                {
                    selected_status = status;
                    selected_response = Some(resp);
                } else {
                    // Try to find a response with a matching condition
                    for (status_code, response_def) in &route_match.endpoint.responses {
                        if let Some(ref condition) = response_def.condition {
                            // Create template context for condition evaluation
                            let state_guard = state.read().await;
                            let request_context = RequestContext::from_request_data(
                                method.to_string(),
                                relative_path.clone(),
                                query_params.clone(),
                                headers.clone(),
                                request_body.clone(),
                            );

                            let template_context = TemplateContext::new(
                                &state_guard,
                                &route_match.path_params,
                                request_context,
                            );

                            // Evaluate condition
                            match template_engine.render(condition, &template_context) {
                                Ok(result) => {
                                    // Check if condition evaluates to truthy
                                    let is_truthy = !result.trim().is_empty()
                                        && result.trim() != "null"
                                        && result.trim() != "false";

                                    if is_truthy {
                                        selected_response = Some(response_def.clone());
                                        selected_status = *status_code;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    log::warn!("Condition evaluation error: {}", e);
                                }
                            }
                        } else {
                            // No condition, use this response as fallback
                            if selected_response.is_none() {
                                selected_response = Some(response_def.clone());
                                selected_status = *status_code;
                            }
                        }
                    }

                    // If no conditional response matched, use default (200 if available)
                    if selected_response.is_none() {
                        if let Some(default_response) = route_match.endpoint.responses.get(&200) {
                            selected_response = Some(default_response.clone());
                            selected_status = 200;
                        } else if let Some((status, response)) =
                            route_match.endpoint.responses.iter().next()
                        {
                            selected_response = Some(response.clone());
                            selected_status = *status;
                        }
                    }
                }

                if let Some(response_def) = selected_response {
                    let request_context = RequestContext::from_request_data(
                        method.to_string(),
                        relative_path.clone(),
                        query_params.clone(),
                        headers.clone(),
                        request_body.clone(),
                    );

                    if let Some(ref script_path) = response_def.script {
                        if let Err(e) = Self::execute_script(
                            script_path.as_path(),
                            &state,
                            &route_match.path_params,
                            &request_context,
                        )
                        .await
                        {
                            log::warn!("Script execution error: {}", e);
                        }
                    }

                    let state_guard = state.read().await;
                    let template_context = TemplateContext::new(
                        &state_guard,
                        &route_match.path_params,
                        request_context,
                    );
                    drop(state_guard);

                    let response_body = response_def.body.clone();
                    let processed_body = if response_body.contains("{{") {
                        match template_engine.render(&response_body, &template_context) {
                            Ok(rendered) => rendered,
                            Err(e) => {
                                log::warn!("Template rendering error: {}", e);
                                response_body
                            }
                        }
                    } else {
                        response_body
                    };

                    if let Some(ref side_effects) = response_def.side_effects {
                        let mut state_guard = state.write().await;
                        for side_effect in side_effects {
                            if let Err(e) = Self::process_side_effect(
                                side_effect,
                                &mut state_guard,
                                &template_context,
                                &template_engine,
                            ) {
                                log::warn!("Side effect processing error: {}", e);
                            }
                        }
                    }

                    let mut response = Response::builder()
                        .status(StatusCode::from_u16(selected_status).unwrap_or(StatusCode::OK))
                        .header("content-type", &response_def.content_type);

                    if let Some(ref headers_map) = response_def.headers {
                        for (key, value) in headers_map {
                            let header_value = if value.contains("{{") {
                                match template_engine.render(value, &template_context) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        log::warn!("Header template rendering error: {}", e);
                                        value.clone()
                                    }
                                }
                            } else {
                                value.clone()
                            };
                            response = response.header(key, header_value);
                        }
                    }

                    // Add CORS headers if enabled
                    if let Some(cfg) = &cors_cfg {
                        let origin_hdr = headers.get("origin").cloned().unwrap_or_default();
                        let allow_origin = if cfg.origins.iter().any(|o| o == "*") {
                            "*".to_string()
                        } else if cfg
                            .origins
                            .iter()
                            .any(|o| o.eq_ignore_ascii_case(&origin_hdr))
                        {
                            origin_hdr.clone()
                        } else {
                            "*".to_string()
                        };
                        let allow_methods = cfg
                            .methods
                            .clone()
                            .map(|v| v.join(", "))
                            .unwrap_or_else(|| {
                                "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string()
                            });
                        let allow_headers = cfg
                            .headers
                            .clone()
                            .map(|v| v.join(", "))
                            .unwrap_or_else(|| "Content-Type, Authorization".to_string());

                        println!("🔧 [{}] Adding CORS headers to response:", service_name);
                        println!("   🌍 Access-Control-Allow-Origin: {}", allow_origin);
                        println!("   🛠️ Access-Control-Allow-Methods: {}", allow_methods);
                        println!("   📋 Access-Control-Allow-Headers: {}", allow_headers);

                        response = response
                            .header("access-control-allow-origin", &allow_origin)
                            .header("access-control-allow-methods", &allow_methods)
                            .header("access-control-allow-headers", &allow_headers);
                    } else {
                        println!(
                            "⚠️ [{}] No CORS config, adding wildcard origin",
                            service_name
                        );
                        response = response.header("access-control-allow-origin", "*");
                    }

                    let final_response = response
                        .body(Full::new(Bytes::from(processed_body)))
                        .map_err(|e| {
                            PulseError::runtime_error(
                                format!("Failed to build response body: {}", e),
                                None::<String>,
                            )
                        })?;

                    println!(
                        "📤 [{}] Sending response with status {}",
                        service_name, selected_status
                    );
                    Self::record_log(
                        &state,
                        &service_name,
                        Some(route_match.endpoint_index),
                        method,
                        path,
                        selected_status,
                    )
                    .await;
                    Ok(final_response)
                } else {
                    // No response definition found
                    let resp = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(
                            r#"{"error": "No response definition found"}"#,
                        )))
                        .map_err(|e| {
                            PulseError::runtime_error(
                                format!("Failed to build error response: {}", e),
                                None::<String>,
                            )
                        })?;
                    Self::record_log(
                        &state,
                        &service_name,
                        Some(route_match.endpoint_index),
                        method,
                        path,
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    )
                    .await;
                    Ok(resp)
                }
            }
            None => {
                // No matching endpoint found
                if let Some(base_url) = proxy_base_url {
                    // Forward request to proxy target
                    let query = parts
                        .uri
                        .query()
                        .map(|q| format!("?{}", q))
                        .unwrap_or_default();
                    let target_url = format!(
                        "{}{}{}",
                        base_url.trim_end_matches('/'),
                        relative_path,
                        query
                    );

                    let client = reqwest::Client::new();
                    let req_method = reqwest::Method::from_bytes(method.as_bytes())
                        .unwrap_or(reqwest::Method::GET);
                    let mut builder = client.request(req_method, target_url);

                    // Copy headers except host
                    for (name, value) in parts.headers.iter() {
                        if name != HOST {
                            if let Ok(v) = value.to_str() {
                                builder = builder.header(name.as_str(), v);
                            }
                        }
                    }

                    if !body_bytes.is_empty() {
                        builder = builder.body(body_bytes.clone());
                    }

                    match builder.send().await {
                        Ok(resp) => {
                            let status = StatusCode::from_u16(resp.status().as_u16())
                                .unwrap_or(StatusCode::OK);
                            let headers = resp.headers().clone();
                            let bytes = resp.bytes().await.unwrap_or_else(|_| Bytes::new());
                            let mut response = Response::builder().status(status);
                            for (name, value) in headers.iter() {
                                if let Ok(v) = value.to_str() {
                                    response = response.header(name.as_str(), v);
                                }
                            }
                            let final_resp = response
                                .body(Full::new(bytes))
                                .map_err(|e| {
                                    PulseError::runtime_error(
                                        format!("Failed to build proxy response: {}", e),
                                        None::<String>,
                                    )
                                })?;
                            Self::record_log(
                                &state,
                                &service_name,
                                None,
                                method,
                                path,
                                status.as_u16(),
                            )
                            .await;
                            Ok(final_resp)
                        }
                        Err(e) => {
                            let resp = Response::builder()
                                .status(StatusCode::BAD_GATEWAY)
                                .header("content-type", "application/json")
                                .body(Full::new(Bytes::from(format!(
                                    r#"{{"error": "Proxy request failed", "details": "{}"}}"#,
                                    e
                                ))))
                                .map_err(|e| {
                                    PulseError::runtime_error(
                                        format!("Failed to build proxy error response: {}", e),
                                        None::<String>,
                                    )
                                })?;
                            Self::record_log(
                                &state,
                                &service_name,
                                None,
                                method,
                                path,
                                StatusCode::BAD_GATEWAY.as_u16(),
                            )
                            .await;
                            Ok(resp)
                        }
                    }
                } else {
                    let resp = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(format!(
                            r#"{{"error": "Endpoint not found", "method": "{}", "path": "{}", "service": "{}"}}"#,
                            method, relative_path, service_name
                        ))))
                        .map_err(|e| {
                            PulseError::runtime_error(
                                format!("Failed to build not found response: {}", e),
                                None::<String>,
                            )
                        })?;
                    Self::record_log(
                        &state,
                        &service_name,
                        None,
                        method,
                        path,
                        StatusCode::NOT_FOUND.as_u16(),
                    )
                    .await;
                    Ok(resp)
                }
            }
        }
    }

    /// Static version of endpoint finding with parameter extraction
    fn find_endpoint_with_params_static(
        endpoints: &[EndpointDefinition],
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Option<RouteMatch> {
        for (index, endpoint) in endpoints.iter().enumerate() {
            if endpoint.method.to_uppercase() == method.to_uppercase()
                && Self::headers_match(endpoint, headers)
            {
                if let Some(path_params) =
                    Self::extract_path_parameters_static(&endpoint.path, path)
                {
                    return Some(RouteMatch {
                        endpoint: endpoint.clone(),
                        endpoint_index: index,
                        path_params,
                    });
                }
            }
        }
        None
    }

    /// Static version of path parameter extraction
    fn extract_path_parameters_static(
        endpoint_path: &str,
        request_path: &str,
    ) -> Option<PathParameters> {
        // Convert endpoint path pattern to regex
        let regex_pattern = Self::endpoint_path_to_regex_static(endpoint_path);

        match Regex::new(&regex_pattern) {
            Ok(regex) => {
                if let Some(captures) = regex.captures(request_path) {
                    let mut params = PathParameters::new();

                    // Extract named parameters
                    for name in regex.capture_names().flatten() {
                        if let Some(matched) = captures.name(name) {
                            params.insert(name.to_string(), matched.as_str().to_string());
                        }
                    }

                    Some(params)
                } else {
                    None
                }
            }
            Err(_) => {
                // Fallback to exact matching if regex compilation fails
                if endpoint_path == request_path {
                    Some(PathParameters::new())
                } else {
                    None
                }
            }
        }
    }

    /// Execute a user-provided script with request context
    async fn execute_script(
        script_path: &Path,
        state: &Arc<RwLock<ServiceState>>,
        path_params: &PathParameters,
        request_context: &RequestContext,
    ) -> PulseResult<()> {
        let script_source = tokio::fs::read_to_string(script_path).await.map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to read script {}: {}", script_path.display(), e),
                Some("Check script path"),
            )
        })?;

        let state_guard = state.read().await;
        let context = serde_json::json!({
            "request": {
                "method": request_context.method.clone(),
                "path": request_context.path.clone(),
                "query": request_context.query.clone(),
                "headers": request_context.headers.clone(),
                "body": request_context.body.clone(),
            },
            "params": path_params.all().clone(),
            "fixtures": state_guard.all_fixtures().clone(),
            "runtime": state_guard.all_runtime_data().clone(),
        });
        drop(state_guard);
        let ctx_json = serde_json::to_string(&context)?;

        let result = tokio::task::spawn_blocking(move || -> PulseResult<serde_json::Value> {
            let mut runtime = JsRuntime::new(RuntimeOptions::default());
            runtime
                .execute_script("<init>", format!("globalThis.ctx = {};", ctx_json))
                .map_err(|e| {
                    PulseError::runtime_error(format!("Script init error: {}", e), None::<String>)
                })?;
            runtime
                .execute_script(
                    "user_script",
                    format!(
                        "globalThis.result = (function(ctx){{ {} }})(ctx);",
                        script_source
                    ),
                )
                .map_err(|e| {
                    PulseError::runtime_error(
                        format!("Script execution error: {}", e),
                        None::<String>,
                    )
                })?;
            deno_core::futures::executor::block_on(
                runtime.run_event_loop(deno_core::PollEventLoopOptions::default()),
            )
            .map_err(|e| {
                PulseError::runtime_error(format!("Script event loop error: {}", e), None::<String>)
            })?;
            let value = runtime.execute_script("<result>", "result").map_err(|e| {
                PulseError::runtime_error(format!("Script result error: {}", e), None::<String>)
            })?;
            let scope = &mut runtime.handle_scope();
            let result_str = value.open(scope).to_rust_string_lossy(scope);
            let result: serde_json::Value = serde_json::from_str(&result_str).map_err(|e| {
                PulseError::runtime_error(
                    format!("Script result serialization error: {}", e),
                    Some("Ensure script returns an object".to_string()),
                )
            })?;
            Ok(result)
        })
        .await
        .map_err(|e| {
            PulseError::runtime_error(format!("Script thread error: {}", e), None::<String>)
        })??;

        if let serde_json::Value::Object(map) = result {
            let mut state_guard = state.write().await;
            for (k, v) in map {
                state_guard.set_runtime_data(k, v);
            }
        }

        Ok(())
    }

    /// Process a side effect from a response
    fn process_side_effect(
        side_effect: &crate::simulator::config::SideEffect,
        state: &mut ServiceState,
        template_context: &TemplateContext,
        template_engine: &TemplateEngine,
    ) -> PulseResult<()> {
        // Render the side effect value template
        let rendered_value = template_engine.render(&side_effect.value, template_context)?;

        // Parse the rendered value as JSON
        let value: Value = serde_json::from_str(&rendered_value).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to parse side effect value as JSON: {}", e),
                Some("Ensure side effect value templates produce valid JSON"),
            )
        })?;

        match side_effect.action.as_str() {
            "add_to_fixture" => {
                state.add_to_fixture_array(&side_effect.target, value)?;
            }
            "update_fixture" => {
                state.set_fixture(side_effect.target.clone(), value);
            }
            "remove_from_fixture" => {
                state.remove_fixture(&side_effect.target);
            }
            "set_runtime_data" => {
                state.set_runtime_data(side_effect.target.clone(), value);
            }
            "remove_runtime_data" => {
                state.remove_runtime_data(&side_effect.target);
            }
            _ => {
                return Err(PulseError::runtime_error(
                    format!("Unknown side effect action: {}", side_effect.action),
                    Some("Use supported actions: add_to_fixture, update_fixture, remove_from_fixture, set_runtime_data, remove_runtime_data")
                ));
            }
        }

        Ok(())
    }

    /// Static version of endpoint path to regex conversion
    fn endpoint_path_to_regex_static(endpoint_path: &str) -> String {
        let mut result = String::new();
        let mut chars = endpoint_path.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    // Start of parameter - collect parameter name
                    let mut param_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '}' {
                            chars.next(); // consume the '}'
                            break;
                        }
                        if let Some(c) = chars.next() {
                            param_name.push(c);
                        } else {
                            break;
                        }
                    }

                    if !param_name.is_empty() {
                        result.push_str(&format!("(?P<{}>[^/]+)", param_name));
                    }
                }
                // Escape special regex characters
                '.' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '|' | '\\' => {
                    result.push('\\');
                    result.push(ch);
                }
                _ => {
                    result.push(ch);
                }
            }
        }

        // Ensure the pattern matches the entire path
        format!("^{}$", result)
    }

    /// Match a scenario based on query, header, or body conditions
    async fn match_scenario(
        endpoint: &EndpointDefinition,
        state: &Arc<RwLock<ServiceState>>,
        endpoint_index: usize,
        active_scenario: Option<String>,
        query: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body: &Option<Value>,
    ) -> Option<(u16, ResponseDefinition)> {
        if let Some(scenarios) = &endpoint.scenarios {
            // First evaluate explicit conditions
            for scenario in scenarios {
                if let Some(cond) = &scenario.conditions {
                    let mut matches = true;
                    if let Some(q) = &cond.query {
                        for (k, v) in q {
                            if query.get(k) != Some(v) {
                                matches = false;
                                break;
                            }
                        }
                    }
                    if matches {
                        if let Some(h) = &cond.headers {
                            for (k, v) in h {
                                match headers.get(k) {
                                    Some(val) if val.eq_ignore_ascii_case(v) => {}
                                    _ => {
                                        matches = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if matches {
                        if let Some(b) = &cond.body {
                            if let Some(Value::Object(obj)) = body {
                                for (k, v) in b {
                                    if obj.get(k) != Some(v) {
                                        matches = false;
                                        break;
                                    }
                                }
                            } else {
                                matches = false;
                            }
                        }
                    }
                    if matches {
                        return Some((
                            scenario.response.status,
                            scenario.response.definition.clone(),
                        ));
                    }
                }
            }
            // Fallback to manually selected scenario
            if let Some(active) = active_scenario {
                for scenario in scenarios {
                    if let Some(name) = &scenario.name {
                        if *name == active {
                            return Some((
                                scenario.response.status,
                                scenario.response.definition.clone(),
                            ));
                        }
                    }
                }
            }

            // Automatic rotation/random selection for scenarios without conditions or name
            let candidates: Vec<&ScenarioDefinition> = scenarios
                .iter()
                .filter(|s| s.conditions.is_none() && s.name.is_none())
                .collect();
            if !candidates.is_empty() {
                let strategy = candidates[0]
                    .strategy
                    .clone()
                    .unwrap_or(ScenarioStrategy::Sequential);
                let index = {
                    let mut guard = state.write().await;
                    guard.next_response_index(endpoint_index, candidates.len(), strategy)
                };
                let scenario = candidates[index];
                return Some((
                    scenario.response.status,
                    scenario.response.definition.clone(),
                ));
            }
        }
        None
    }

    /// Get service behavior configuration
    pub fn behavior(&self) -> Option<&crate::simulator::config::BehaviorConfig> {
        self.definition.behavior.as_ref()
    }

    /// Check if the service has CORS enabled
    pub fn has_cors(&self) -> bool {
        self.definition
            .server
            .cors
            .as_ref()
            .map(|cors| cors.enabled)
            .unwrap_or(false)
    }

    /// Get CORS configuration
    pub fn cors_config(&self) -> Option<&crate::simulator::config::CorsConfig> {
        self.definition.server.cors.as_ref()
    }

    /// Validate that the service configuration is consistent
    pub fn validate_consistency(&self) -> PulseResult<()> {
        // Check for duplicate endpoint paths with same method
        let mut seen_endpoints = std::collections::HashSet::new();

        for endpoint in &self.definition.endpoints {
            let key = format!("{}:{}", endpoint.method.to_uppercase(), endpoint.path);
            if seen_endpoints.contains(&key) {
                return Err(PulseError::config_error(
                    format!(
                        "Duplicate endpoint found: {} {}",
                        endpoint.method, endpoint.path
                    ),
                    Some("Each endpoint must have a unique method-path combination"),
                ));
            }
            seen_endpoints.insert(key);
        }

        // Validate that referenced models exist if request body schemas are specified
        if let Some(ref models) = self.definition.models {
            for endpoint in &self.definition.endpoints {
                if let Some(ref request_body) = endpoint.request_body {
                    if let Some(ref schema) = request_body.schema {
                        if !models.contains_key(schema) {
                            return Err(PulseError::config_error(
                                format!("Referenced model '{}' not found in service '{}'", 
                                    schema, self.definition.name),
                                Some("Define the model in the models section or remove the schema reference")
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::config::{
        EndpointKind, ResponseDefinition, ScenarioConditions, ScenarioDefinition, ScenarioResponse,
        ServerConfig,
    };
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full};
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{
        Request as HyperRequest, Response as HyperResponse, StatusCode as HyperStatusCode,
    };
    use hyper_util::rt::TokioIo;
    use reqwest::StatusCode as ReqStatusCode;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::RwLock;
    use tokio::time::{sleep, Duration};

    fn create_test_service_definition() -> ServiceDefinition {
        ServiceDefinition {
            name: "test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service".to_string()),
            server: ServerConfig {
                port: Some(8001),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
            },
            models: None,
            fixtures: {
                let mut fixtures = HashMap::new();
                fixtures.insert(
                    "users".to_string(),
                    serde_json::json!([
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]),
                );
                Some(fixtures)
            },
            bucket: None,
            endpoints: vec![
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users".to_string(),
                    header_match: None,
                    description: Some("Get all users".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body: "{{ fixtures.users }}".to_string(),
                                script: None,
                                headers: None,
                                side_effects: None,
                            },
                        );
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users/1".to_string(),
                    header_match: None,
                    description: Some("Get user by ID".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body: r#"{"id": 1, "name": "Alice"}"#.to_string(),
                                script: None,
                                headers: None,
                                side_effects: None,
                            },
                        );
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
            ],
            graphql: None,
            behavior: None,
        }
    }

    #[tokio::test]
    async fn test_service_instance_creation() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8003, storage, tx).unwrap(); // Use different port

        assert_eq!(service.name(), "test-service");
        assert_eq!(service.port(), 8003);
        assert_eq!(service.base_path(), "/api/v1");
        assert_eq!(service.endpoints_count(), 2);
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_service_start_stop() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service =
            ServiceInstance::new(definition, 8002, storage, tx).unwrap(); // Use different port to avoid conflicts

        assert!(!service.is_running());

        service.start().await.unwrap();
        assert!(service.is_running());

        service.stop().await.unwrap();
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_service_state_management() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8004, storage, tx).unwrap(); // Use different port

        // Test fixture access
        let users = service.get_state("users").await.unwrap();
        assert!(users.is_array());

        // Test runtime data
        service
            .update_state("test_key", serde_json::json!("test_value"))
            .await;
        let value = service.get_state("test_key").await.unwrap();
        assert_eq!(value, serde_json::json!("test_value"));
    }

    #[tokio::test]
    async fn test_fixture_array_operations() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8010, storage, tx).unwrap();

        // Test adding to fixture array
        let new_user = serde_json::json!({"id": 3, "name": "Charlie"});
        service
            .add_to_fixture_array("users", new_user)
            .await
            .unwrap();

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array.len(), 3);
        assert_eq!(users_array[2]["name"], "Charlie");

        // Test updating array item by index
        let updated_user = serde_json::json!({"id": 3, "name": "Charles"});
        service
            .update_fixture_array_item("users", 2, updated_user)
            .await
            .unwrap();

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array[2]["name"], "Charles");

        // Test removing from array by index
        let removed_user = service.remove_from_fixture_array("users", 1).await.unwrap();
        assert_eq!(removed_user["name"], "Bob");

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array.len(), 2);
    }

    #[tokio::test]
    async fn test_fixture_array_operations_by_field() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8011, storage, tx).unwrap();

        // Test updating by field value
        let updated_user = serde_json::json!({"id": 1, "name": "Alice Updated"});
        let found = service
            .update_fixture_array_item_by_field("users", "id", &serde_json::json!(1), updated_user)
            .await
            .unwrap();
        assert!(found);

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array[0]["name"], "Alice Updated");

        // Test removing by field value
        let removed_user = service
            .remove_fixture_array_item_by_field("users", "id", &serde_json::json!(2))
            .await
            .unwrap();
        assert!(removed_user.is_some());
        assert_eq!(removed_user.unwrap()["name"], "Bob");

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array.len(), 1);
    }

    #[tokio::test]
    async fn test_fixture_reset() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8012, storage, tx).unwrap();

        // Modify fixtures
        service
            .add_to_fixture_array("users", serde_json::json!({"id": 3, "name": "Charlie"}))
            .await
            .unwrap();
        service
            .update_fixture("new_fixture", serde_json::json!("test"))
            .await;

        // Verify modifications
        let users = service.get_fixtures().await;
        assert_eq!(users.get("users").unwrap().as_array().unwrap().len(), 3);
        assert!(users.contains_key("new_fixture"));

        // Reset fixtures
        service.reset_fixtures().await;

        // Verify reset
        let users = service.get_fixtures().await;
        assert_eq!(users.get("users").unwrap().as_array().unwrap().len(), 2);
        assert!(!users.contains_key("new_fixture"));
    }

    #[tokio::test]
    async fn test_runtime_data_management() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8013, storage, tx).unwrap();

        // Test setting runtime data
        service
            .set_runtime_data("session_id", serde_json::json!("abc123"))
            .await;
        service
            .set_runtime_data("user_count", serde_json::json!(42))
            .await;

        // Test getting runtime data
        let session_id = service.get_runtime_data("session_id").await.unwrap();
        assert_eq!(session_id, serde_json::json!("abc123"));

        // Test checking existence
        assert!(service.has_runtime_data("session_id").await);
        assert!(!service.has_runtime_data("nonexistent").await);

        // Test removing runtime data
        let removed = service.remove_runtime_data("session_id").await.unwrap();
        assert_eq!(removed, serde_json::json!("abc123"));
        assert!(!service.has_runtime_data("session_id").await);

        // Test clearing all runtime data
        service.clear_runtime_data().await;
        assert!(!service.has_runtime_data("user_count").await);

        let (fixture_count, runtime_count) = service.get_state_info().await;
        assert_eq!(fixture_count, 1); // users fixture
        assert_eq!(runtime_count, 0);
    }

    #[tokio::test]
    async fn test_side_effects_processing() {
        use crate::simulator::config::SideEffect;
        use crate::simulator::template::{RequestContext, TemplateContext, TemplateEngine};

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let mut state = ServiceState::new(
            Some({
                let mut fixtures = HashMap::new();
                fixtures.insert("users".to_string(), serde_json::json!([]));
                fixtures
            }),
            None,
            storage,
            None,
        );

        let template_engine = TemplateEngine::new().unwrap();
        let params = PathParameters::new();
        let request_context = RequestContext::from_request_data(
            "POST".to_string(),
            "/users".to_string(),
            HashMap::new(),
            HashMap::new(),
            Some(serde_json::json!({"id": 1, "name": "Alice"})),
        );
        let template_context = TemplateContext::new(&state, &params, request_context);

        // Test add_to_fixture side effect
        let side_effect = SideEffect {
            action: "add_to_fixture".to_string(),
            target: "users".to_string(),
            value: r#"{"id": 1, "name": "Alice"}"#.to_string(),
        };

        ServiceInstance::process_side_effect(
            &side_effect,
            &mut state,
            &template_context,
            &template_engine,
        )
        .unwrap();

        let users = state.get_fixture("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["name"], "Alice");

        // Test set_runtime_data side effect
        let side_effect = SideEffect {
            action: "set_runtime_data".to_string(),
            target: "last_user_id".to_string(),
            value: "1".to_string(),
        };

        ServiceInstance::process_side_effect(
            &side_effect,
            &mut state,
            &template_context,
            &template_engine,
        )
        .unwrap();

        let last_id = state.get_runtime_data("last_user_id").unwrap();
        assert_eq!(last_id, &serde_json::json!(1));
    }

    #[tokio::test]
    async fn test_endpoint_finding() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8005, storage, tx).unwrap(); // Use different port

        let headers = HashMap::new();
        let endpoint = service.find_endpoint("GET", "/users", &headers);
        assert!(endpoint.is_some());
        assert_eq!(endpoint.unwrap().path, "/users");

        let endpoint = service.find_endpoint("POST", "/users", &headers);
        assert!(endpoint.is_none());

        let endpoint = service.find_endpoint("get", "/users", &headers); // Case insensitive
        assert!(endpoint.is_some());
    }

    #[tokio::test]
    async fn test_path_parameter_extraction() {
        let definition = create_test_service_definition_with_params();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8008, storage, tx).unwrap();

        // Test parameter extraction
        let headers = HashMap::new();
        let route_match = service.find_endpoint_with_params("GET", "/users/123", &headers);
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(route_match.endpoint.path, "/users/{id}");
        assert_eq!(route_match.path_params.get("id"), Some(&"123".to_string()));

        // Test multiple parameters
        let route_match =
            service.find_endpoint_with_params("GET", "/users/123/orders/456", &headers);
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(
            route_match.path_params.get("userId"),
            Some(&"123".to_string())
        );
        assert_eq!(
            route_match.path_params.get("orderId"),
            Some(&"456".to_string())
        );

        // Test no match
        let route_match = service.find_endpoint_with_params("GET", "/products/123", &headers);
        assert!(route_match.is_none());
    }

    #[tokio::test]
    async fn test_endpoint_header_matching() {
        let definition = create_test_service_definition_with_header_match();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8010, storage, tx).unwrap();

        let mut headers = HashMap::new();
        // Missing header should not match
        let endpoint = service.find_endpoint("GET", "/protected", &headers);
        assert!(endpoint.is_none());

        // Correct header should match
        headers.insert("x-api-key".to_string(), "secret".to_string());
        let endpoint = service.find_endpoint("GET", "/protected", &headers);
        assert!(endpoint.is_some());
    }

    #[test]
    fn test_endpoint_path_to_regex() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8009, storage, tx).unwrap();

        // Test simple parameter
        let regex = service.endpoint_path_to_regex("/users/{id}");
        assert_eq!(regex, "^/users/(?P<id>[^/]+)$");

        // Test multiple parameters
        let regex = service.endpoint_path_to_regex("/users/{userId}/orders/{orderId}");
        assert_eq!(
            regex,
            "^/users/(?P<userId>[^/]+)/orders/(?P<orderId>[^/]+)$"
        );

        // Test no parameters
        let regex = service.endpoint_path_to_regex("/users");
        assert_eq!(regex, "^/users$");
    }

    #[tokio::test]
    async fn test_template_processing_with_params() {
        use crate::simulator::template::{RequestContext, TemplateContext, TemplateEngine};

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let mut state = ServiceState::new(None, None, storage, None);
        state.set_fixture(
            "users".to_string(),
            serde_json::json!([{"id": 1, "name": "Alice"}]),
        );

        let mut params = PathParameters::new();
        params.insert("id".to_string(), "123".to_string());

        let request_context = RequestContext::from_request_data(
            "GET".to_string(),
            "/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        );

        let template_context = TemplateContext::new(&state, &params, request_context);
        let engine = TemplateEngine::new().unwrap();

        let template = r#"{"user_id": "{{params.id}}", "users": {{json fixtures.users}}}"#;
        let result = engine.render(template, &template_context);

        // Debug print to see what we got
        println!("Template result: {:?}", result);

        let result = result.unwrap();
        assert!(result.contains(r#""user_id": "123""#));
    }

    fn create_test_service_definition_with_params() -> ServiceDefinition {
        ServiceDefinition {
            name: "test-service-params".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service with parameters".to_string()),
            server: ServerConfig {
                port: Some(8001),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
            },
            models: None,
            fixtures: {
                let mut fixtures = HashMap::new();
                fixtures.insert(
                    "users".to_string(),
                    serde_json::json!([
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]),
                );
                Some(fixtures)
            },
            bucket: None,
            endpoints: vec![
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users/{id}".to_string(),
                    header_match: None,
                    description: Some("Get user by ID".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body:
                                    r#"{"id": "{{ params.id }}", "name": "User {{ params.id }}"}"#
                                        .to_string(),
                                script: None,
                                headers: None,
                                side_effects: None,
                            },
                        );
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users/{userId}/orders/{orderId}".to_string(),
                    header_match: None,
                    description: Some("Get user order".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(200, ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"userId": "{{ params.userId }}", "orderId": "{{ params.orderId }}"}"#.to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        });
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
            ],
            graphql: None,
            behavior: None,
        }
    }

    fn create_test_service_definition_with_header_match() -> ServiceDefinition {
        let mut definition = create_test_service_definition();
        // Add an endpoint that requires a specific header
        definition.endpoints.push(EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/protected".to_string(),
            description: Some("Requires header".to_string()),
            header_match: Some(HashMap::from([(
                "x-api-key".to_string(),
                "secret".to_string(),
            )])),
            parameters: None,
            request_body: None,
            responses: {
                let mut responses = HashMap::new();
                responses.insert(
                    200,
                    ResponseDefinition {
                        condition: None,
                        content_type: "application/json".to_string(),
                        body: "{\"status\":\"ok\"}".to_string(),
                        script: None,
                        headers: None,
                        side_effects: None,
                    },
                );
                responses
            },
            scenarios: None,
            stream: None,
        });
        definition
    }

    #[tokio::test]
    async fn test_service_validation() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8006, storage, tx).unwrap(); // Use different port

        let result = service.validate_consistency();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_duplicate_endpoint_validation() {
        let mut definition = create_test_service_definition();

        // Add duplicate endpoint
        definition.endpoints.push(EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/users".to_string(), // Duplicate path with same method
            description: Some("Duplicate endpoint".to_string()),
            header_match: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: None,
            stream: None,
        });

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8007, storage, tx).unwrap(); // Use different port
        let result = service.validate_consistency();
        assert!(result.is_err());
    }

    #[test]
    fn test_endpoint_path_to_regex_handles_unclosed_brace() {
        let pattern = ServiceInstance::endpoint_path_to_regex_static("/users/{id");
        assert!(pattern.contains("users"));
    }

    #[tokio::test]
    async fn test_scenario_matching() {
        // Build endpoint with various scenarios
        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/test".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: Some(vec![
                ScenarioDefinition {
                    name: Some("query".to_string()),
                    conditions: Some(ScenarioConditions {
                        query: Some(HashMap::from([("mode".to_string(), "1".to_string())])),
                        headers: None,
                        body: None,
                    }),
                    response: ScenarioResponse {
                        status: 200,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"result\":\"query\"}".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
                ScenarioDefinition {
                    name: Some("header".to_string()),
                    conditions: Some(ScenarioConditions {
                        query: None,
                        headers: Some(HashMap::from([("x-scn".to_string(), "hdr".to_string())])),
                        body: None,
                    }),
                    response: ScenarioResponse {
                        status: 201,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"result\":\"header\"}".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
                ScenarioDefinition {
                    name: Some("body".to_string()),
                    conditions: Some(ScenarioConditions {
                        query: None,
                        headers: None,
                        body: Some(HashMap::from([(
                            "kind".to_string(),
                            serde_json::json!("b"),
                        )])),
                    }),
                    response: ScenarioResponse {
                        status: 202,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"result\":\"body\"}".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
                ScenarioDefinition {
                    name: Some("error".to_string()),
                    conditions: None,
                    response: ScenarioResponse {
                        status: 500,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"error\":\"forced\"}".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
            ]),
            stream: None,
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let state = Arc::new(RwLock::new(ServiceState::new(None, None, storage, None)));

        // Query condition
        let mut query = HashMap::new();
        query.insert("mode".to_string(), "1".to_string());
        let res = ServiceInstance::match_scenario(
            &endpoint,
            &state,
            0,
            None,
            &query,
            &HashMap::new(),
            &None,
        )
        .await;
        assert_eq!(res.unwrap().0, 200);

        // Header condition
        let mut headers = HashMap::new();
        headers.insert("x-scn".to_string(), "hdr".to_string());
        let res = ServiceInstance::match_scenario(
            &endpoint,
            &state,
            0,
            None,
            &HashMap::new(),
            &headers,
            &None,
        )
        .await;
        assert_eq!(res.unwrap().0, 201);

        // Body condition
        let body = Some(serde_json::json!({"kind": "b"}));
        let res = ServiceInstance::match_scenario(
            &endpoint,
            &state,
            0,
            None,
            &HashMap::new(),
            &HashMap::new(),
            &body,
        )
        .await;
        assert_eq!(res.unwrap().0, 202);

        // Active scenario fallback
        let res = ServiceInstance::match_scenario(
            &endpoint,
            &state,
            0,
            Some("error".to_string()),
            &HashMap::new(),
            &HashMap::new(),
            &None,
        )
        .await;
        assert_eq!(res.unwrap().0, 500);
    }

    #[tokio::test]
    async fn test_scenario_rotation_sequential() {
        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/rotate".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: Some(vec![
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Sequential),
                    response: ScenarioResponse {
                        status: 200,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "a".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Sequential),
                    response: ScenarioResponse {
                        status: 201,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "b".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Sequential),
                    response: ScenarioResponse {
                        status: 202,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "c".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
            ]),
            stream: None,
        };
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let state = Arc::new(RwLock::new(ServiceState::new(None, None, storage, None)));
        let mut results = Vec::new();
        for _ in 0..4 {
            let res = ServiceInstance::match_scenario(
                &endpoint,
                &state,
                0,
                None,
                &HashMap::new(),
                &HashMap::new(),
                &None,
            )
            .await
            .unwrap()
            .0;
            results.push(res);
        }
        assert_eq!(results, vec![200, 201, 202, 200]);
    }

    #[tokio::test]
    async fn test_scenario_rotation_random() {
        use std::collections::HashSet;

        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/random".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: Some(vec![
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Random),
                    response: ScenarioResponse {
                        status: 200,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "ok".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Random),
                    response: ScenarioResponse {
                        status: 500,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "err".to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
            ]),
            stream: None,
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let state = Arc::new(RwLock::new(ServiceState::new(None, None, storage, None)));
        let mut statuses = HashSet::new();
        for _ in 0..20 {
            let res = ServiceInstance::match_scenario(
                &endpoint,
                &state,
                0,
                None,
                &HashMap::new(),
                &HashMap::new(),
                &None,
            )
            .await
            .unwrap()
            .0;
            statuses.insert(res);
        }
        assert_eq!(statuses.len(), 2);
    }

    fn spawn_upstream_server(port: u16) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
            if let Ok((stream, _)) = listener.accept().await {
                let io = TokioIo::new(stream);
                let service = service_fn(|req: HyperRequest<hyper::body::Incoming>| async move {
                    let (parts, body) = req.into_parts();
                    let header_val = parts
                        .headers
                        .get("x-test-header")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");
                    let bytes = BodyExt::collect(body).await.unwrap().to_bytes();
                    let body_str = String::from_utf8_lossy(&bytes);
                    let resp_body = format!("header={};body={}", header_val, body_str);
                    Ok::<_, Infallible>(
                        HyperResponse::builder()
                            .status(HyperStatusCode::OK)
                            .header("x-test-header", header_val)
                            .body(Full::new(Bytes::from(resp_body)))
                            .unwrap(),
                    )
                });
                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Upstream server error: {}", e);
                }
            }
        })
    }

    #[tokio::test]
    async fn test_proxy_forwards_unmatched_requests() {
        let upstream_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };
        let upstream_handle = spawn_upstream_server(upstream_port);

        let mut definition = create_test_service_definition();
        definition.server.proxy_base_url = Some(format!("http://127.0.0.1:{}", upstream_port));

        let service_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, service_port, storage, tx).unwrap();
        service.start().await.unwrap();
        sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/api/v1/unknown", service_port);
        let resp = client
            .post(&url)
            .header("x-test-header", "abc")
            .body("hello")
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), ReqStatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("x-test-header")
                .and_then(|v| v.to_str().ok()),
            Some("abc")
        );
        let body = resp.text().await.unwrap();
        assert_eq!(body, "header=abc;body=hello");

        service.stop().await.unwrap();
        upstream_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_proxy_disabled_returns_not_found() {
        let mut definition = create_test_service_definition();
        definition.server.proxy_base_url = None; // ensure proxy disabled

        let service_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, service_port, storage, tx).unwrap();
        service.start().await.unwrap();
        sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/api/v1/unknown", service_port);
        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), ReqStatusCode::NOT_FOUND);

        service.stop().await.unwrap();
    }

    #[test]
    fn test_service_state_operations() {
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let mut state = ServiceState::new(None, None, storage, None);

        // Test runtime data
        state.set_runtime_data("key1".to_string(), serde_json::json!("value1"));
        assert_eq!(
            state.get_runtime_data("key1"),
            Some(&serde_json::json!("value1"))
        );

        // Test fixtures
        state.set_fixture("fixture1".to_string(), serde_json::json!({"data": "test"}));
        assert_eq!(
            state.get_fixture("fixture1"),
            Some(&serde_json::json!({"data": "test"}))
        );

        // Test non-existent keys
        assert_eq!(state.get_runtime_data("nonexistent"), None);
        assert_eq!(state.get_fixture("nonexistent"), None);

        // Test data bucket
        let bucket = state.bucket();
        bucket.set("foo".to_string(), serde_json::json!(123));
        assert_eq!(bucket.get("foo"), Some(serde_json::json!(123)));
    }
}
