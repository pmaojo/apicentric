use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};
#[cfg(feature = "file-watch")]
use tokio::sync::mpsc;

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::{
    config::{ConfigLoader, ServiceDefinition, SimulatorConfig},
    log::RequestLogEntry,
    registry::ServiceRegistry,
    route_registry::RouteRegistry,
    ConfigChange,
};

#[cfg(feature = "file-watch")]
use crate::simulator::watcher::ConfigWatcher;
use tracing::info;

/// Trait for managing simulator lifecycle.
#[async_trait]
pub trait Lifecycle: Send + Sync {
    async fn start(&self) -> ApicentricResult<()>;
    async fn stop(&self) -> ApicentricResult<()>;
}

/// Handles starting and stopping of the simulator.
pub struct SimulatorLifecycle<R: RouteRegistry + Send + Sync> {
    pub(crate) config: SimulatorConfig,
    pub(crate) service_registry: Arc<RwLock<ServiceRegistry>>,
    pub(crate) route_registry: Arc<RwLock<R>>,
    pub(crate) config_loader: ConfigLoader,
    pub(crate) is_active: Arc<RwLock<bool>>,
    #[cfg(feature = "file-watch")]
    pub(crate) config_watcher: Arc<RwLock<Option<ConfigWatcher>>>,
    pub(crate) log_sender: broadcast::Sender<RequestLogEntry>,
}

impl<R: RouteRegistry + Send + Sync> SimulatorLifecycle<R> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: SimulatorConfig,
        service_registry: Arc<RwLock<ServiceRegistry>>,
        route_registry: Arc<RwLock<R>>,
        config_loader: ConfigLoader,
        is_active: Arc<RwLock<bool>>,
        #[cfg(feature = "file-watch")]
        config_watcher: Arc<RwLock<Option<ConfigWatcher>>>,
        log_sender: broadcast::Sender<RequestLogEntry>,
    ) -> Self {
        Self {
            config,
            service_registry,
            route_registry,
            config_loader,
            is_active,
            #[cfg(feature = "file-watch")]
            config_watcher,
            log_sender,
        }
    }
}

#[async_trait]
impl<R: RouteRegistry + Send + Sync + 'static> Lifecycle for SimulatorLifecycle<R> {
    #[tracing::instrument(skip(self), fields(simulator_enabled = self.config.enabled))]
    async fn start(&self) -> ApicentricResult<()> {
        if !self.config.enabled {
            return Err(ApicentricError::config_error(
                "API simulator is not enabled",
                Some("Enable in configuration"),
            ));
        }

        let mut is_active = self.is_active.write().await;
        if *is_active {
            return Err(ApicentricError::runtime_error(
                "API simulator is already running",
                None::<String>,
            ));
        }

        // Load service definitions
        let services = self.config_loader.load_all_services()?;

        if services.is_empty() {
            return Err(ApicentricError::config_error(
                "No service definitions found",
                Some("Add YAML service definition files to the services directory"),
            ));
        }

        // Register and start services
        let mut registry = self.service_registry.write().await;
        let mut router = self.route_registry.write().await;

        for service_def in services {
            let service_name = service_def.name.clone();
            let base_path = service_def
                .server
                .as_ref()
                .map(|s| s.base_path.clone())
                .unwrap_or_else(|| "/".to_string());

            registry.register_service(service_def.clone()).await?;
            router.register_service(&service_name, &base_path);
        }

        registry.start_all_services().await?;
        let service_count = registry.services_count();
        drop(registry);
        drop(router);

        *is_active = true;

        info!(
            target: "simulator",
            service_count = service_count,
            "API Simulator started"
        );

        #[cfg(feature = "file-watch")]
        {
            // Spawn configuration watcher for automatic reloads
            let (tx, mut rx) = mpsc::channel(16);
            let watcher = ConfigWatcher::new(self.config.services_dir.clone(), tx).map_err(|e| {
                ApicentricError::runtime_error(
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
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn stop(&self) -> ApicentricResult<()> {
        let mut is_active = self.is_active.write().await;
        if !*is_active {
            return Ok(()); // Already stopped
        }

        // Stop all services
        let mut registry = self.service_registry.write().await;
        registry.stop_all_services().await?;

        // Clear router mappings
        let mut router = self.route_registry.write().await;
        router.clear_all();

        *is_active = false;

        info!(target: "simulator", "API Simulator stopped");

        Ok(())
    }
}

impl<R: RouteRegistry + Send + Sync> Clone for SimulatorLifecycle<R> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            service_registry: self.service_registry.clone(),
            route_registry: self.route_registry.clone(),
            config_loader: self.config_loader.clone(),
            is_active: self.is_active.clone(),
            #[cfg(feature = "file-watch")]
            config_watcher: self.config_watcher.clone(),
            log_sender: self.log_sender.clone(),
        }
    }
}

impl<R: RouteRegistry + Send + Sync> SimulatorLifecycle<R> {
    pub async fn apply_remote_service(&self, service: ServiceDefinition) -> ApicentricResult<()> {
        let mut registry = self.service_registry.write().await;
        registry.register_service(service.clone()).await?;
        let mut router = self.route_registry.write().await;
        let base_path = service
            .server
            .as_ref()
            .map(|s| s.base_path.clone())
            .unwrap_or_else(|| "/".to_string());
        router.register_service(&service.name, &base_path);
        registry.start_all_services().await?;
        Ok(())
    }

    pub async fn handle_config_change(&self, change: ConfigChange) -> ApicentricResult<()> {
        match change {
            ConfigChange::ServiceAdded(service_name) => {
                println!("📁 Service added: {}", service_name);
            }
            ConfigChange::ServiceModified(service_name) => {
                println!("📝 Service modified: {}", service_name);
            }
            ConfigChange::ServiceRemoved(service_name) => {
                println!("🗑️ Service removed: {}", service_name);
            }
        }

        if *self.is_active.read().await {
            self.reload_services_internal().await?;
        }

        Ok(())
    }

    pub async fn reload_services_internal(&self) -> ApicentricResult<()> {
        let services = self.config_loader.load_all_services()?;
        let mut registry = self.service_registry.write().await;
        let mut router = self.route_registry.write().await;

        registry.clear_all_services().await?;
        router.clear_all();

        for service_def in services {
            let service_name = service_def.name.clone();
            let base_path = service_def
                .server
                .as_ref()
                .map(|s| s.base_path.clone())
                .unwrap_or_else(|| "/".to_string());
            registry.register_service(service_def.clone()).await?;
            router.register_service(&service_name, &base_path);
        }

        registry.start_all_services().await?;
        Ok(())
    }
}
