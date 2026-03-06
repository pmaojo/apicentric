//! Service Instance - Individual service implementation with state management

pub mod graphql;
pub mod http_handler;
pub mod http_server;
pub mod response_processor;
pub mod router;
pub mod routing;
pub mod scenario;
pub mod scenario_matcher;
pub mod state;
pub mod state_service;
#[cfg(feature = "iot")]
pub mod twin_runner;

#[cfg(test)]
pub mod tests;

pub use graphql::*;
pub use http_server::HttpServer;
pub use router::{DefaultRouter, RequestRouter};
pub use routing::*;
pub use scenario::ScenarioService;
pub use state::*;
pub use state_service::StateService;

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::config::{EndpointDefinition, ServiceDefinition};
use crate::simulator::log::RequestLogEntry;
use crate::simulator::scripting::ScriptingEngine;
use crate::simulator::template::TemplateEngine;
use crate::storage::Storage;
use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

/// Individual service instance with HTTP server capabilities
pub struct ServiceInstance {
    definition: Arc<StdRwLock<ServiceDefinition>>,
    port: u16,
    state: Arc<RwLock<ServiceState>>,
    template_engine: Arc<TemplateEngine>,
    scripting_engine: Arc<ScriptingEngine>,
    server_handle: Option<JoinHandle<()>>,
    #[cfg(feature = "iot")]
    twin_handle: Option<JoinHandle<()>>,
    is_running: bool,
    active_scenario: Arc<RwLock<Option<String>>>,
    graphql: Option<Arc<GraphQLMocks>>,
    storage: Arc<dyn Storage>,
}

impl ServiceInstance {
    /// Create a new service instance
    pub fn new(
        definition: ServiceDefinition,
        port: u16,
        storage: Arc<dyn Storage>,
        log_sender: broadcast::Sender<RequestLogEntry>,
    ) -> ApicentricResult<Self> {
        let fixtures = definition.fixtures.clone();
        let bucket = definition.bucket.clone();
        let graphql_cfg = definition.graphql.clone();

        let definition = Arc::new(StdRwLock::new(definition));

        let state = ServiceState::new(fixtures, bucket, Arc::clone(&storage), Some(log_sender));

        // Initialize template engine and register bucket helpers
        let mut template_engine = TemplateEngine::new()?;
        template_engine.register_bucket_helpers(state.bucket())?;

        let scripting_engine = Arc::new(ScriptingEngine::new());

        let graphql = if let Some(gql_cfg) = graphql_cfg {
            Some(Arc::new(load_graphql_mocks(&gql_cfg)?))
        } else {
            None
        };

        let saved_definition = { definition.read().unwrap().clone() };
        let _ = storage.save_service(&saved_definition);

        Ok(Self {
            definition,
            port,
            state: Arc::new(RwLock::new(state)),
            template_engine: Arc::new(template_engine),
            scripting_engine,
            server_handle: None,
            #[cfg(feature = "iot")]
            twin_handle: None,
            is_running: false,
            active_scenario: Arc::new(RwLock::new(None)),
            graphql,
            storage,
        })
    }

    /// Start the service (HTTP server or Digital Twin runner)
    pub async fn start(&mut self) -> ApicentricResult<()> {
        if self.is_running {
            return Err(ApicentricError::runtime_error(
                format!(
                    "Service '{}' is already running",
                    self.definition.read().unwrap().name
                ),
                None::<String>,
            ));
        }

        #[cfg(feature = "iot")]
        {
            let twin_def = self.definition.read().unwrap().twin.clone();
            if let Some(twin_def) = twin_def {
                self.start_twin_runner(twin_def).await?;
            }
        }

        let (server_cfg, service_name) = {
            let definition_guard = self.definition.read().unwrap();
            (
                definition_guard.server.clone(),
                definition_guard.name.clone(),
            )
        };

        // Standard HTTP service
        let server_cfg = match server_cfg {
            Some(cfg) => cfg,
            None => {
                if self.is_running {
                    // Twin started, no server config needed
                    return Ok(());
                } else {
                    return Err(ApicentricError::config_error(
                        format!("Service '{}' has no server configuration", service_name),
                        Some("Add a 'server' block or a 'twin' block to the configuration"),
                    ));
                }
            }
        };

        let base_path = server_cfg.base_path.clone();

        // Create TCP listener for the service
        // Use 0.0.0.0 to bind to all interfaces (required for LAN access on mobile)
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to bind to port {}: {}", self.port, e),
                Some("Port may already be in use or unavailable"),
            )
        })?;

        // Clone necessary data for the server task
        let definition = Arc::clone(&self.definition);
        let state = Arc::clone(&self.state);
        let template_engine = Arc::clone(&self.template_engine);
        let scripting_engine = Arc::clone(&self.scripting_engine);
        let active_scenario = Arc::clone(&self.active_scenario);
        let graphql = self.graphql.clone();
        let storage = Arc::clone(&self.storage);

        // Spawn the HTTP server task
        let server_handle = tokio::spawn(async move {
            Self::record_log(
                &state,
                &service_name,
                None,
                "SYSTEM",
                &format!("Started on port {} at {}", addr.port(), base_path),
                200,
                None,
            )
            .await;

            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let io = TokioIo::new(stream);
                        let service_name_for_request = service_name.clone();
                        let service_name_for_error = service_name.clone();
                        let definition = Arc::clone(&definition);
                        let state = Arc::clone(&state);
                        let template_engine = Arc::clone(&template_engine);
                        let scripting_engine = Arc::clone(&scripting_engine);
                        let scenario_cfg_outer = Arc::clone(&active_scenario);
                        let graphql_cfg_outer = graphql.clone();
                        let storage = Arc::clone(&storage);

                        tokio::task::spawn(async move {
                            let service = service_fn(move |req| {
                                let service_name = service_name_for_request.clone();
                                let definition = Arc::clone(&definition);
                                let state = Arc::clone(&state);
                                let template_engine = Arc::clone(&template_engine);
                                let scripting_engine = Arc::clone(&scripting_engine);
                                let scenario_cfg = Arc::clone(&scenario_cfg_outer);
                                let graphql_cfg = graphql_cfg_outer.clone();
                                let storage = Arc::clone(&storage);

                                async move {
                                    match http_handler::HttpHandler::handle_request(
                                        req,
                                        Arc::clone(&definition),
                                        state,
                                        template_engine,
                                        scripting_engine,
                                        scenario_cfg,
                                        graphql_cfg,
                                        storage,
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
                                                )))) {
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
                                    service_name_for_error, err
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
    pub async fn stop(&mut self) -> ApicentricResult<()> {
        if !self.is_running {
            return Ok(()); // Already stopped
        }

        // Stop the server if it's running
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }

        #[cfg(feature = "iot")]
        if let Some(handle) = self.twin_handle.take() {
            handle.abort();
        }

        // Wait a moment for graceful shutdown
        if self.is_running {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        self.is_running = false;

        println!(
            "🛑 Stopped service '{}'",
            self.definition.read().unwrap().name
        );

        Ok(())
    }

    /// Handle a single HTTP request (for external use)
    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> ApicentricResult<Response<Full<Bytes>>> {
        if !self.is_running {
            return Err(ApicentricError::runtime_error(
                format!(
                    "Service '{}' is not running",
                    self.definition.read().unwrap().name
                ),
                Some("Start the service before handling requests"),
            ));
        }

        http_handler::HttpHandler::handle_request(
            req,
            Arc::clone(&self.definition),
            Arc::clone(&self.state),
            Arc::clone(&self.template_engine),
            Arc::clone(&self.scripting_engine),
            Arc::clone(&self.active_scenario),
            self.graphql.clone(),
            Arc::clone(&self.storage),
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
    pub fn base_path(&self) -> String {
        self.definition
            .read()
            .unwrap()
            .server
            .as_ref()
            .map(|s| s.base_path.clone())
            .unwrap_or_else(|| "/".to_string())
    }

    /// Get the service name
    pub fn name(&self) -> String {
        self.definition.read().unwrap().name.clone()
    }

    /// Get the number of endpoints defined for this service
    pub fn endpoints_count(&self) -> usize {
        self.definition
            .read()
            .unwrap()
            .endpoints
            .as_ref()
            .map(|e| e.len())
            .unwrap_or(0)
    }

    /// Get all endpoint definitions
    pub fn endpoints(&self) -> Vec<EndpointDefinition> {
        self.definition
            .read()
            .unwrap()
            .endpoints
            .clone()
            .unwrap_or_default()
    }

    /// Get the service definition
    pub fn definition(&self) -> ServiceDefinition {
        self.definition.read().unwrap().clone()
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
        payload: Option<String>,
    ) {
        let mut guard = state.write().await;
        guard.add_log_entry(RequestLogEntry::new(
            service.to_string(),
            endpoint,
            method.to_string(),
            path.to_string(),
            status,
            payload,
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
    pub async fn add_to_fixture_array(
        &self,
        fixture_key: &str,
        item: Value,
    ) -> ApicentricResult<()> {
        let mut state = self.state.write().await;
        state.add_to_fixture_array(fixture_key, item)
    }

    /// Remove an item from a fixture array by index
    pub async fn remove_from_fixture_array(
        &self,
        fixture_key: &str,
        index: usize,
    ) -> ApicentricResult<Value> {
        let mut state = self.state.write().await;
        state.remove_from_fixture_array(fixture_key, index)
    }

    /// Update an item in a fixture array by index
    pub async fn update_fixture_array_item(
        &self,
        fixture_key: &str,
        index: usize,
        item: Value,
    ) -> ApicentricResult<()> {
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
    ) -> ApicentricResult<bool> {
        let mut state = self.state.write().await;
        state.update_fixture_array_item_by_field(fixture_key, field, field_value, new_item)
    }

    /// Find and remove an item from a fixture array by field value
    pub async fn remove_fixture_array_item_by_field(
        &self,
        fixture_key: &str,
        field: &str,
        field_value: &Value,
    ) -> ApicentricResult<Option<Value>> {
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
        let def = self.definition.read().unwrap();
        crate::simulator::ServiceInfo {
            name: def.name.clone(),
            port: self.port,
            base_path: def
                .server
                .as_ref()
                .map(|s| s.base_path.clone())
                .unwrap_or_else(|| "/".to_string()),
            endpoints_count: def.endpoints.as_ref().map(|e| e.len()).unwrap_or(0),
            is_running: self.is_running,
            version: def.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
            definition: serde_yaml::to_string(&*def).unwrap_or_default(),
            endpoints: def.endpoints.clone().unwrap_or_default(),
        }
    }

    /// Find an endpoint by method, path and headers with parameter extraction
    pub fn find_endpoint_with_params(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Option<RouteMatch> {
        let definition = self.definition.read().unwrap();
        if let Some(endpoints) = &definition.endpoints {
            for (index, endpoint) in endpoints.iter().enumerate() {
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
        }
        None
    }

    /// Find an endpoint by method, path and headers (legacy reference)
    pub fn find_endpoint(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Option<EndpointDefinition> {
        let definition = self.definition.read().unwrap();
        if let Some(endpoints) = &definition.endpoints {
            for endpoint in endpoints {
                if endpoint.method.to_uppercase() == method.to_uppercase()
                    && Self::headers_match(endpoint, headers)
                    && self.extract_path_parameters(&endpoint.path, path).is_some()
                {
                    return Some(endpoint.clone());
                }
            }
        }
        None
    }

    /// Check if request headers satisfy an endpoint's header_match criteria
    pub(crate) fn headers_match(
        endpoint: &EndpointDefinition,
        headers: &HashMap<String, String>,
    ) -> bool {
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
    pub(crate) fn extract_path_parameters(
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
    pub(crate) fn endpoint_path_to_regex(&self, endpoint_path: &str) -> String {
        Self::endpoint_path_to_regex_static(endpoint_path)
    }

    /// Static version of endpoint finding with parameter extraction
    pub(crate) fn find_endpoint_with_params_static(
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
    pub(crate) fn extract_path_parameters_static(
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

    pub(crate) fn build_recorded_endpoint(
        method: &str,
        relative_path: &str,
    ) -> (EndpointDefinition, String) {
        let (normalized_path, parameters) = Self::normalize_recorded_path(relative_path);

        let mut responses = HashMap::new();
        responses.insert(
            200,
            crate::simulator::config::ResponseDefinition {
                condition: None,
                content_type: "application/json".to_string(),
                body: serde_json::json!({
                    "message": "Respuesta placeholder generada automáticamente",
                    "method": method,
                    "path": normalized_path,
                })
                .to_string(),
                schema: None,
                script: None,
                headers: None,
                side_effects: None,
            },
        );

        let endpoint = EndpointDefinition {
            kind: crate::simulator::config::EndpointKind::Http,
            method: method.to_uppercase(),
            path: normalized_path.clone(),
            header_match: None,
            description: Some("Endpoint generado automáticamente desde tráfico real".to_string()),
            parameters: if parameters.is_empty() {
                None
            } else {
                Some(parameters)
            },
            request_body: None,
            responses,
            scenarios: None,
            stream: None,
        };

        (endpoint, normalized_path)
    }

    pub(crate) fn normalize_recorded_path(
        path: &str,
    ) -> (String, Vec<crate::simulator::config::ParameterDefinition>) {
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        let mut params = Vec::new();
        let mut normalized_segments = Vec::new();
        let mut param_index = 1;

        for segment in segments {
            if Self::is_dynamic_segment(segment) {
                let name = format!("param{}", param_index);
                param_index += 1;
                normalized_segments.push(format!("{{{}}}", name));
                params.push(crate::simulator::config::ParameterDefinition {
                    name: name.clone(),
                    location: crate::simulator::config::ParameterLocation::Path,
                    param_type: "string".to_string(),
                    required: true,
                    description: Some(format!(
                        "Valor capturado automáticamente desde '{}'.",
                        segment
                    )),
                });
            } else {
                normalized_segments.push(segment.to_string());
            }
        }

        let normalized_path = if normalized_segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", normalized_segments.join("/"))
        };

        (normalized_path, params)
    }

    pub(crate) fn is_dynamic_segment(segment: &str) -> bool {
        if segment.is_empty() {
            return false;
        }

        segment.parse::<i64>().is_ok()
            || segment.parse::<u64>().is_ok()
            || Self::looks_like_uuid(segment)
            || (segment.chars().any(|c| c.is_ascii_digit())
                && segment.chars().any(|c| c.is_ascii_alphabetic())
                && segment.len() > 3)
            || segment
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
    }

    pub(crate) fn looks_like_uuid(segment: &str) -> bool {
        let stripped: String = segment.chars().filter(|c| *c != '-').collect();
        (segment.len() == 36 || segment.len() == 32)
            && !stripped.is_empty()
            && stripped.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Static version of endpoint path to regex conversion
    pub(crate) fn endpoint_path_to_regex_static(endpoint_path: &str) -> String {
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

    /// Get service behavior configuration
    pub fn behavior(&self) -> Option<crate::simulator::config::BehaviorConfig> {
        self.definition.read().unwrap().behavior.clone()
    }

    /// Get CORS configuration
    pub fn cors_config(&self) -> Option<crate::simulator::config::CorsConfig> {
        self.definition
            .read()
            .unwrap()
            .server
            .as_ref()
            .and_then(|s| s.cors.clone())
    }

    /// Check if the service has CORS enabled
    pub fn has_cors(&self) -> bool {
        self.cors_config().map(|cors| cors.enabled).unwrap_or(false)
    }

    /// Validate that the service definition is consistent
    pub fn validate_consistency(&self) -> ApicentricResult<()> {
        let definition = self.definition.read().unwrap();
        // Check for duplicate endpoint paths with same method
        let mut seen_endpoints = std::collections::HashSet::new();

        if let Some(endpoints) = &definition.endpoints {
            for endpoint in endpoints {
                let key = format!("{}:{}", endpoint.method.to_uppercase(), endpoint.path);
                if seen_endpoints.contains(&key) {
                    return Err(ApicentricError::config_error(
                        format!(
                            "Duplicate endpoint found: {} {}",
                            endpoint.method, endpoint.path
                        ),
                        Some("Each endpoint must have a unique method-path combination"),
                    ));
                }
                seen_endpoints.insert(key);
            }
        }

        // Validate that referenced models exist if request body schemas are specified
        if let Some(ref models) = definition.models {
            if let Some(endpoints) = &definition.endpoints {
                for endpoint in endpoints {
                    if let Some(ref request_body) = endpoint.request_body {
                        if let Some(ref schema) = request_body.schema {
                            if !models.contains_key(schema) {
                                return Err(ApicentricError::config_error(
                                    format!("Referenced model '{}' not found in service '{}'",
                                        schema, definition.name),
                                    Some("Define the model in the models section or remove the schema reference")
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// Tests are now in tests/mod.rs
