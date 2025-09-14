//! API Simulator Manager - Central coordinator for the simulator functionality

use crate::errors::{PulseError, PulseResult};
use crate::simulator::{
    config::{
        ConfigLoader, EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig,
        ServiceDefinition, SimulatorConfig,
    },
    registry::ServiceRegistry,
    router::RequestRouter,
    watcher::ConfigWatcher,
    ConfigChange, SimulatorStatus,
};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex, RwLock};

/// Central coordinator for the API simulator functionality
pub struct ApiSimulatorManager {
    config: SimulatorConfig,
    service_registry: Arc<RwLock<ServiceRegistry>>,
    request_router: Arc<RwLock<RequestRouter>>,
    config_loader: ConfigLoader,
    is_active: Arc<RwLock<bool>>,
    config_watcher: Arc<RwLock<Option<ConfigWatcher>>>,
}

impl Clone for ApiSimulatorManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            service_registry: self.service_registry.clone(),
            request_router: self.request_router.clone(),
            config_loader: self.config_loader.clone(),
            is_active: self.is_active.clone(),
            config_watcher: self.config_watcher.clone(),
        }
    }
}

impl ApiSimulatorManager {
    /// Create a new API simulator manager
    pub fn new(config: SimulatorConfig) -> Self {
        let config_loader = ConfigLoader::new(config.services_dir.clone());
        let service_registry =
            Arc::new(RwLock::new(ServiceRegistry::new(config.port_range.clone())));
        let request_router = Arc::new(RwLock::new(RequestRouter::new()));
        let is_active = Arc::new(RwLock::new(false));
        let config_watcher = Arc::new(RwLock::new(None));

        Self {
            config,
            service_registry,
            request_router,
            config_loader,
            is_active,
            config_watcher,
        }
    }

    /// Start the API simulator
    pub async fn start(&self) -> PulseResult<()> {
        if !self.config.enabled {
            return Err(PulseError::config_error(
                "API simulator is not enabled",
                Some("Set PULSE_API_SIMULATOR=true or enable in configuration"),
            ));
        }

        let mut is_active = self.is_active.write().await;
        if *is_active {
            return Err(PulseError::runtime_error(
                "API simulator is already running",
                None::<String>,
            ));
        }

        // Load service definitions
        let services = self.config_loader.load_all_services()?;

        if services.is_empty() {
            return Err(PulseError::config_error(
                "No service definitions found",
                Some("Add YAML service definition files to the services directory"),
            ));
        }

        // Register and start services
        let mut registry = self.service_registry.write().await;
        let mut router = self.request_router.write().await;

        for service_def in services {
            let service_name = service_def.name.clone();
            let base_path = service_def.server.base_path.clone();

            // Register service in registry
            registry.register_service(service_def).await?;

            // Register routes in router
            router.register_service_routes(&service_name, &base_path);
        }

        // Start all registered services
        registry.start_all_services().await?;

        *is_active = true;

        log::info!(
            "API Simulator started with {} services",
            registry.services_count()
        );
        // Spawn configuration watcher for automatic reloads
        let (tx, mut rx) = mpsc::channel(16);
        let watcher = ConfigWatcher::new(self.config.services_dir.clone(), tx).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to watch services directory: {}", e),
                None::<String>,
            )
        })?;

        {
            let mut guard = self.config_watcher.write().await;
            *guard = Some(watcher);
        }

        let manager_clone = self.clone();
        tokio::spawn(async move {
            while let Some(change) = rx.recv().await {
                if let Err(e) = manager_clone.handle_config_change(change).await {
                    eprintln!("Error handling config change: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop the API simulator
    pub async fn stop(&self) -> PulseResult<()> {
        let mut is_active = self.is_active.write().await;
        if !*is_active {
            return Ok(()); // Already stopped
        }

        // Stop all services
        let mut registry = self.service_registry.write().await;
        registry.stop_all_services().await?;

        // Clear router mappings
        let mut router = self.request_router.write().await;
        router.clear_all_routes();

        *is_active = false;

        log::info!("API Simulator stopped");

        Ok(())
    }

    /// Reload service configurations
    pub async fn reload_services(&self) -> PulseResult<()> {
        if !*self.is_active.read().await {
            return Err(PulseError::runtime_error(
                "Cannot reload services: simulator is not running",
                Some("Start the simulator first"),
            ));
        }

        log::info!("Reloading service configurations...");

        // Load updated service definitions
        let services = self.config_loader.load_all_services()?;

        let mut registry = self.service_registry.write().await;
        let mut router = self.request_router.write().await;

        // Stop and clear all current services and routes
        registry.clear_all_services().await?;
        router.clear_all_routes();

        // Re-register and start services with new configurations
        for service_def in services {
            let service_name = service_def.name.clone();
            let base_path = service_def.server.base_path.clone();

            registry.register_service(service_def).await?;
            router.register_service_routes(&service_name, &base_path);
        }

        registry.start_all_services().await?;

        log::info!("Service configurations reloaded successfully");

        Ok(())
    }

    /// Set the active scenario for all services
    pub async fn set_scenario(&self, scenario: Option<String>) -> PulseResult<()> {
        let registry = self.service_registry.read().await;
        registry.set_scenario_all(scenario).await;
        Ok(())
    }

    /// Get current simulator status
    pub async fn get_status(&self) -> SimulatorStatus {
        let is_active = *self.is_active.read().await;
        let registry = self.service_registry.read().await;

        let services_count = registry.services_count();
        let active_services = registry.list_services().await;

        SimulatorStatus {
            is_active,
            services_count,
            active_services,
        }
    }

    /// Check if the simulator is currently active
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Get the service registry for external access
    pub fn service_registry(&self) -> &Arc<RwLock<ServiceRegistry>> {
        &self.service_registry
    }

    /// Get the request router for external access
    pub fn request_router(&self) -> &Arc<RwLock<RequestRouter>> {
        &self.request_router
    }

    /// Run a reverse proxy that records requests/responses and generates
    /// service definitions. All traffic is forwarded to `target` and captured
    /// into YAML files under `output_dir`. Blocks until CTRL+C is received.
    pub async fn record(&self, target: &str, output_dir: PathBuf) -> PulseResult<()> {
        let port = self.config.port_range.start;
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let connector = HttpConnector::new();
        let client: Client<HttpConnector, Full<Bytes>> =
            Client::builder(TokioExecutor::new()).build(connector);
        let endpoints: Arc<Mutex<HashMap<(String, String), EndpointDefinition>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to bind recording proxy: {}", e),
                None::<String>,
            )
        })?;

        println!(
            "🔴 Recording proxy listening on http://{} forwarding to {}",
            addr, target
        );

        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (stream, _) = res.map_err(|e| PulseError::runtime_error(format!("Accept error: {}", e), None::<String>))?;
                    let io = TokioIo::new(stream);
                    let client = client.clone();
                    let target = target.to_string();
                    let endpoints = endpoints.clone();
                    tokio::spawn(async move {
                        let service = service_fn(move |req: Request<Incoming>| {
                            let client = client.clone();
                            let target = target.clone();
                            let endpoints = endpoints.clone();
                            async move {
                                let method = req.method().clone();
                                let headers = req.headers().clone();
                                let path = req.uri().path().to_string();
                                let path_and_query = req
                                    .uri()
                                    .path_and_query()
                                    .map(|pq| pq.as_str().to_string())
                                    .unwrap_or_else(|| path.clone());

                                let req_body = match BodyExt::collect(req.into_body()).await {
                                    Ok(col) => col.to_bytes(),
                                    Err(e) => {
                                        let mut err_resp: Response<Full<Bytes>> =
                                            Response::new(Full::from(Bytes::from(e.to_string())));
                                        *err_resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
                                        return Ok::<_, Infallible>(err_resp);
                                    }
                                };

                                let uri: Uri = format!("{}{}", target, path_and_query)
                                    .parse()
                                    .unwrap();

                                let mut fwd_req = Request::new(Full::from(req_body.clone()));
                                *fwd_req.method_mut() = method.clone();
                                *fwd_req.uri_mut() = uri;
                                *fwd_req.headers_mut() = headers.clone();

                                let resp = match client.request(fwd_req).await {
                                    Ok(r) => r,
                                    Err(e) => {
                                        let mut err_resp = Response::new(Full::from(Bytes::from(e.to_string())));
                                        *err_resp.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                                        return Ok::<_, Infallible>(err_resp);
                                    }
                                };
                                let (parts, body) = resp.into_parts();
                                let resp_bytes = match BodyExt::collect(body).await {
                                    Ok(col) => col.to_bytes(),
                                    Err(e) => {
                                        let mut err_resp: Response<Full<Bytes>> =
                                            Response::new(Full::from(Bytes::from(e.to_string())));
                                        *err_resp.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                                        return Ok::<_, Infallible>(err_resp);
                                    }
                                };

                                let content_type = parts
                                    .headers
                                    .get(hyper::header::CONTENT_TYPE)
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("application/json")
                                    .to_string();
                                {
                                    let mut map = endpoints.lock().await;
                                    let key = (method.to_string(), path.clone());
                                    let entry = map.entry(key).or_insert_with(|| EndpointDefinition {
                                        kind: EndpointKind::Http,
                                        method: method.to_string(),
                                        path: path.clone(),
                                        header_match: None,
                                        description: None,
                                        parameters: None,
                                        request_body: None,
                                        responses: HashMap::new(),
                                        scenarios: None,
                                        stream: None,
                                    });
                                    entry.responses.insert(
                                        parts.status.as_u16(),
                                        ResponseDefinition {
                                            condition: None,
                                            content_type: content_type.clone(),
                                            body: String::from_utf8_lossy(&resp_bytes).into(),
                                            headers: None,
                                            side_effects: None,
                                        },
                                    );
                                }

                                let mut client_resp: Response<Full<Bytes>> =
                                    Response::new(Full::from(resp_bytes));
                                *client_resp.status_mut() = parts.status;
                                *client_resp.headers_mut() = parts.headers;
                                Ok::<_, Infallible>(client_resp)
                            }
                        });
                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                            eprintln!("Proxy connection error: {err}");
                        }
                    });
                },
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        let map = endpoints.lock().await;
        let service = ServiceDefinition {
            name: "recorded_service".to_string(),
            version: None,
            description: Some("Recorded service".to_string()),
            server: ServerConfig {
                port: None,
                base_path: "/".to_string(),
                proxy_base_url: None,
                cors: None,
            },
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: map.values().cloned().collect(),
            graphql: None,
            behavior: None,
        };

        std::fs::create_dir_all(&output_dir).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to create output directory: {}", e),
                None::<String>,
            )
        })?;
        let yaml = serde_yaml::to_string(&service).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to serialize service definition: {}", e),
                None::<String>,
            )
        })?;
        let path = output_dir.join("recorded_service.yaml");
        std::fs::write(&path, yaml).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to write service file: {}", e),
                None::<String>,
            )
        })?;
        println!("✅ Recorded interactions saved to {}", path.display());
        Ok(())
    }

    /// Validate service configurations without starting
    pub fn validate_configurations(&self) -> PulseResult<Vec<String>> {
        let services = self.config_loader.load_all_services()?;
        let service_names: Vec<String> = services.iter().map(|s| s.name.clone()).collect();

        println!(
            "✅ Validated {} service configurations",
            service_names.len()
        );
        for name in &service_names {
            println!("  - {}", name);
        }

        Ok(service_names)
    }

    /// Handle configuration change events (for future hot-reload implementation)
    pub async fn handle_config_change(&self, change: ConfigChange) -> PulseResult<()> {
        match change {
            ConfigChange::ServiceAdded(service_name) => {
                println!("📁 Service added: {}", service_name);
                // Future: Load and register the new service
            }
            ConfigChange::ServiceModified(service_name) => {
                println!("📝 Service modified: {}", service_name);
                // Future: Reload the specific service
            }
            ConfigChange::ServiceRemoved(service_name) => {
                println!("🗑️ Service removed: {}", service_name);
                // Future: Unregister and stop the service
            }
        }

        // For now, just trigger a full reload
        if *self.is_active.read().await {
            self.reload_services().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::config::PortRange;
    use tempfile::TempDir;
    use tokio;

    fn create_test_config() -> SimulatorConfig {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");
        std::fs::create_dir_all(&services_dir).unwrap();

        SimulatorConfig {
            enabled: true,
            services_dir,
            port_range: PortRange {
                start: 9000,
                end: 9999,
            },
            global_behavior: None,
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let config = create_test_config();
        let manager = ApiSimulatorManager::new(config);

        assert!(!manager.is_active().await);

        let status = manager.get_status().await;
        assert!(!status.is_active);
        assert_eq!(status.services_count, 0);
    }

    #[tokio::test]
    async fn test_start_without_services() {
        let config = create_test_config();
        let manager = ApiSimulatorManager::new(config);

        let result = manager.start().await;
        assert!(result.is_err());

        // Should contain error about no service definitions
        let error_msg = result.unwrap_err().to_string();

        assert!(
            error_msg.contains("No service definitions found")
                || error_msg.contains("No valid service definitions")
                || error_msg.contains("Services directory does not exist")
        );
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let config = create_test_config();
        let manager = ApiSimulatorManager::new(config);

        // Should not error when stopping a non-running simulator
        let result = manager.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reload_when_not_running() {
        let config = create_test_config();
        let manager = ApiSimulatorManager::new(config);

        let result = manager.reload_services().await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("simulator is not running"));
    }

    #[tokio::test]
    async fn test_config_change_handling() {
        let config = create_test_config();
        let manager = ApiSimulatorManager::new(config);

        // Should handle config changes gracefully even when not running
        let result = manager
            .handle_config_change(ConfigChange::ServiceAdded("test".to_string()))
            .await;
        assert!(result.is_ok());
    }
}
