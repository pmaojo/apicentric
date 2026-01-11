//! API Simulator Manager - Central coordinator for the simulator functionality

#[cfg(feature = "p2p")]
use crate::collab::crdt::{CrdtMessage, ServiceCrdt};
use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::{
    admin_server::AdminServer,
    config::{ConfigLoader, ServiceDefinition, SimulatorConfig},
    lifecycle::{Lifecycle, SimulatorLifecycle},
    log::RequestLogEntry,
    recording_proxy::{ProxyRecorder, RecordingProxy},
    registry::ServiceRegistry,
    router::RequestRouter,
    watcher::ConfigWatcher,
    ConfigChange, SimulatorStatus,
};
use crate::storage::sqlite::SqliteStorage;
#[cfg(feature = "p2p")]
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::info;

/// Central coordinator for the API simulator functionality
pub struct ApiSimulatorManager {
    config: SimulatorConfig,
    service_registry: Arc<RwLock<ServiceRegistry>>,
    route_registry: Arc<RwLock<RequestRouter>>,
    config_loader: ConfigLoader,
    is_active: Arc<RwLock<bool>>,
    #[allow(dead_code)]
    p2p_enabled: Arc<RwLock<bool>>,
    #[allow(dead_code)]
    collab_sender: Arc<RwLock<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    #[cfg(feature = "p2p")]
    crdts: Arc<RwLock<HashMap<String, ServiceCrdt>>>,
    log_sender: broadcast::Sender<RequestLogEntry>,
    lifecycle: SimulatorLifecycle<RequestRouter>,
    recorder: ProxyRecorder,
    admin_server: Arc<RwLock<AdminServer>>,
}

impl ApiSimulatorManager {
    /// Create a new API simulator manager
    pub fn new(config: SimulatorConfig) -> Self {
        let config_loader = ConfigLoader::new(config.services_dir.clone());
        let storage = Arc::new(
            SqliteStorage::init_db(config.db_path.clone())
                .expect("failed to initialize sqlite storage"),
        );
        let (log_sender, _) = broadcast::channel(100);
        let service_registry = Arc::new(RwLock::new(ServiceRegistry::new(
            config.port_range.clone(),
            storage,
            log_sender.clone(),
        )));
        let route_registry = Arc::new(RwLock::new(RequestRouter::new()));
        let is_active = Arc::new(RwLock::new(false));
        let config_watcher: Arc<RwLock<Option<ConfigWatcher>>> = Arc::new(RwLock::new(None));
        let p2p_enabled = Arc::new(RwLock::new(false));
        let collab_sender = Arc::new(RwLock::new(None));
        #[cfg(feature = "p2p")]
        let crdts = Arc::new(RwLock::new(HashMap::new()));

        let lifecycle = SimulatorLifecycle::new(
            config.clone(),
            service_registry.clone(),
            route_registry.clone(),
            config_loader.clone(),
            is_active.clone(),
            config_watcher.clone(),
            p2p_enabled.clone(),
            collab_sender.clone(),
            #[cfg(feature = "p2p")]
            crdts.clone(),
            log_sender.clone(),
        );
        let recorder = ProxyRecorder;
        let admin_server = Arc::new(RwLock::new(AdminServer::new(service_registry.clone())));

        Self {
            config,
            service_registry,
            route_registry,
            config_loader,
            is_active,
            p2p_enabled,
            collab_sender,
            #[cfg(feature = "p2p")]
            crdts,
            log_sender,
            lifecycle,
            recorder,
            admin_server,
        }
    }

    /// Update database path for persistent storage
    pub async fn set_db_path<P: AsRef<std::path::Path>>(&self, path: P) -> ApicentricResult<()> {
        let storage = Arc::new(SqliteStorage::init_db(path)?);
        let mut reg = self.service_registry.write().await;
        reg.set_storage(storage);
        Ok(())
    }

    /// Enable or disable peer-to-peer collaboration.
    pub async fn enable_p2p(&self, enabled: bool) {
        let mut flag = self.p2p_enabled.write().await;
        *flag = enabled;
    }

    /// Subscribe to log events
    pub fn subscribe_logs(&self) -> broadcast::Receiver<RequestLogEntry> {
        self.log_sender.subscribe()
    }

    /// Start the API simulator
    pub async fn start(&self) -> ApicentricResult<()> {
        if let Some(port) = self.config.admin_port {
            let mut admin_server = self.admin_server.write().await;
            admin_server.start(port).await;
        }
        self.lifecycle.start().await
    }

    /// Stop the API simulator
    pub async fn stop(&self) -> ApicentricResult<()> {
        if self.config.admin_port.is_some() {
            let mut admin_server = self.admin_server.write().await;
            admin_server.stop().await;
        }
        self.lifecycle.stop().await
    }

    /// Reload service definitions
    pub async fn reload_services(&self) -> ApicentricResult<()> {
        if !*self.is_active.read().await {
            return Err(ApicentricError::runtime_error(
                "Cannot reload services: simulator is not running",
                Some("Start the simulator first"),
            ));
        }

        info!(target: "simulator", "Reloading service configurations");

        self.lifecycle.reload_services_internal().await
    }

    /// Apply a service definition and update CRDT state.
    #[cfg(feature = "p2p")]
    pub async fn apply_service_definition(
        &self,
        service_def: ServiceDefinition,
    ) -> ApicentricResult<()> {
        let service_name = service_def.name.clone();
        self.lifecycle
            .apply_remote_service(service_def.clone())
            .await?;
        {
            let mut crdts = self.crdts.write().await;
            if let Some(doc) = crdts.get_mut(&service_name) {
                doc.apply_local_change(service_def.clone());
            } else {
                crdts.insert(service_name.clone(), ServiceCrdt::new(service_def.clone()));
            }

            if *self.p2p_enabled.read().await {
                if let Some(tx) = self.collab_sender.read().await.clone() {
                    if let Some(doc) = crdts.get_mut(&service_name) {
                        if let Ok(data) = serde_json::to_vec(&CrdtMessage {
                            name: service_name.clone(),
                            data: doc.encode(),
                        }) {
                            let _ = tx.send(data);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Apply a service definition (P2P feature disabled)
    #[cfg(not(feature = "p2p"))]
    pub async fn apply_service_definition(
        &self,
        service_def: ServiceDefinition,
    ) -> ApicentricResult<()> {
        self.lifecycle.apply_remote_service(service_def).await
    }

    /// Apply a YAML service definition string to the running simulator and CRDT.
    pub async fn apply_service_yaml(&self, yaml: &str) -> ApicentricResult<String> {
        let def: ServiceDefinition = serde_yaml::from_str(yaml).map_err(|e| {
            ApicentricError::validation_error(
                format!("Invalid service YAML: {}", e),
                None::<String>,
                None::<String>,
            )
        })?;
        let service_name = def.name.clone();
        self.apply_service_definition(def).await?;
        Ok(service_name)
    }

    /// Set the active scenario for all services
    pub async fn set_scenario(&self, scenario: Option<String>) -> ApicentricResult<()> {
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

    /// Get the route registry for external access
    pub fn route_registry(&self) -> &Arc<RwLock<RequestRouter>> {
        &self.route_registry
    }

    /// Run a reverse proxy that records requests/responses.
    pub async fn record(&self, target: &str, output_dir: PathBuf) -> ApicentricResult<()> {
        self.recorder
            .record(target, output_dir, self.config.port_range.start)
            .await
    }

    /// Validate service definitions without starting
    pub fn validate_configurations(&self) -> ApicentricResult<Vec<String>> {
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
    pub async fn handle_config_change(&self, change: ConfigChange) -> ApicentricResult<()> {
        self.lifecycle.handle_config_change(change).await
    }

    /// Start a specific service by name
    pub async fn start_service(&self, service_name: &str) -> ApicentricResult<()> {
        let registry = self.service_registry.read().await;

        if let Some(service_arc) = registry.get_service(service_name) {
            let mut service = service_arc.write().await;

            if service.is_running() {
                return Err(ApicentricError::runtime_error(
                    format!("Service '{}' is already running", service_name),
                    Some("Stop the service first or use --force to restart"),
                ));
            }

            service.start().await?;
            info!(
                target: "simulator",
                service = %service_name,
                "Service started"
            );
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                format!("Service '{}' not found", service_name),
                Some("Check that the service is registered"),
            ))
        }
    }

    /// Stop a specific service by name
    pub async fn stop_service(&self, service_name: &str) -> ApicentricResult<()> {
        let registry = self.service_registry.read().await;

        if let Some(service_arc) = registry.get_service(service_name) {
            let mut service = service_arc.write().await;

            if !service.is_running() {
                return Err(ApicentricError::runtime_error(
                    format!("Service '{}' is not running", service_name),
                    Some("Start the service first with 'apicentric simulator start'"),
                ));
            }

            service.stop().await?;
            info!(
                target: "simulator",
                service = %service_name,
                "Service stopped"
            );
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                format!("Service '{}' not found", service_name),
                Some("Check that the service is registered"),
            ))
        }
    }

    /// Load services from the configured directory
    pub async fn load_services(&self) -> ApicentricResult<()> {
        let services = self.config_loader.load_all_services()?;
        let mut registry = self.service_registry.write().await;
        for service in services {
            registry.register_service(service).await?;
        }
        Ok(())
    }
}
