#[cfg(feature = "p2p")]
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc, RwLock};

#[cfg(feature = "p2p")]
use crate::collab::{crdt::{ServiceCrdt, CrdtMessage}, p2p};
use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::{
    config::{ConfigLoader, SimulatorConfig, ServiceDefinition},
    log::RequestLogEntry,
    registry::ServiceRegistry,
    route_registry::RouteRegistry,
    watcher::ConfigWatcher,
    ConfigChange,
};

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
    pub(crate) config_watcher: Arc<RwLock<Option<ConfigWatcher>>>,
    pub(crate) p2p_enabled: Arc<RwLock<bool>>,
    pub(crate) collab_sender: Arc<RwLock<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    #[cfg(feature = "p2p")]
    pub(crate) crdts: Arc<RwLock<HashMap<String, ServiceCrdt>>>,
    pub(crate) log_sender: broadcast::Sender<RequestLogEntry>,
}

impl<R: RouteRegistry + Send + Sync> SimulatorLifecycle<R> {
    pub fn new(
        config: SimulatorConfig,
        service_registry: Arc<RwLock<ServiceRegistry>>,
        route_registry: Arc<RwLock<R>>,
        config_loader: ConfigLoader,
        is_active: Arc<RwLock<bool>>,
        config_watcher: Arc<RwLock<Option<ConfigWatcher>>>,
        p2p_enabled: Arc<RwLock<bool>>,
        collab_sender: Arc<RwLock<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
        #[cfg(feature = "p2p")]
        crdts: Arc<RwLock<HashMap<String, ServiceCrdt>>>,
        log_sender: broadcast::Sender<RequestLogEntry>,
    ) -> Self {
        Self {
            config,
            service_registry,
            route_registry,
            config_loader,
            is_active,
            config_watcher,
            p2p_enabled,
            collab_sender,
            #[cfg(feature = "p2p")]
            crdts,
            log_sender,
        }
    }
}

#[async_trait]
impl<R: RouteRegistry + Send + Sync + 'static> Lifecycle for SimulatorLifecycle<R> {
    async fn start(&self) -> ApicentricResult<()> {
        if !self.config.enabled {
            return Err(ApicentricError::config_error(
                "API simulator is not enabled",
                Some("Set PULSE_API_SIMULATOR=true or enable in configuration"),
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
        
        #[cfg(feature = "p2p")]
        let mut crdts_map = self.crdts.write().await;

        for service_def in services {
            let service_name = service_def.name.clone();
            let base_path = service_def.server.base_path.clone();

            registry.register_service(service_def.clone()).await?;
            router.register_service(&service_name, &base_path);
            
            #[cfg(feature = "p2p")]
            crdts_map.insert(service_name, ServiceCrdt::new(service_def));
        }
        
        #[cfg(feature = "p2p")]
        drop(crdts_map);

        registry.start_all_services().await?;
        let service_count = registry.services_count();
        drop(registry);
        drop(router);

        *is_active = true;

        log::info!(
            "API Simulator started with {} services",
            service_count
        );
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

        // Start peer-to-peer collaboration if enabled.
        #[cfg(feature = "p2p")]
        if *self.p2p_enabled.read().await {
            match p2p::spawn().await {
                Ok((tx, mut rx_net)) => {
                    {
                        let mut guard = self.collab_sender.write().await;
                        *guard = Some(tx.clone());
                    }

                    // Broadcast initial state for all services.
                    let mut crdts = self.crdts.write().await;
                    for (name, doc) in crdts.iter_mut() {
                        if let Ok(data) = serde_json::to_vec(&CrdtMessage {
                            name: name.clone(),
                            data: doc.encode(),
                        }) {
                            let _ = tx.send(data);
                        }
                    }
                    drop(crdts);

                    let manager_clone = self.clone();
                    let crdts_map = self.crdts.clone();
                    tokio::spawn(async move {
                        while let Some(data) = rx_net.recv().await {
                            if let Ok(msg) = serde_json::from_slice::<CrdtMessage>(&data) {
                                let mut map = crdts_map.write().await;
                                if let Some(existing) = map.get_mut(&msg.name) {
                                    existing.merge_bytes(&msg.data);
                                } else if let Some(new_doc) = ServiceCrdt::from_bytes(&msg.data) {
                                    map.insert(msg.name.clone(), new_doc);
                                }
                                if let Some(doc) = map.get(&msg.name) {
                                    let service = doc.to_service();
                                    drop(map);
                                    if let Err(err) = manager_clone.apply_remote_service(service).await {
                                        eprintln!("Failed to apply remote update: {}", err);
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failed to start P2P session: {}", e),
            }
        }

        Ok(())
    }

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

        log::info!("API Simulator stopped");

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
            config_watcher: self.config_watcher.clone(),
            p2p_enabled: self.p2p_enabled.clone(),
            collab_sender: self.collab_sender.clone(),
            #[cfg(feature = "p2p")]
            crdts: self.crdts.clone(),
            log_sender: self.log_sender.clone(),
        }
    }
}

impl<R: RouteRegistry + Send + Sync> SimulatorLifecycle<R> {
    pub async fn apply_remote_service(&self, service: ServiceDefinition) -> ApicentricResult<()> {
        let mut registry = self.service_registry.write().await;
        registry.register_service(service.clone()).await?;
        let mut router = self.route_registry.write().await;
        router.register_service(&service.name, &service.server.base_path);
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

        #[cfg(feature = "p2p")]
        {
            let mut crdts_map = self.crdts.write().await;
            crdts_map.clear();
            for service_def in services.clone() {
                let service_name = service_def.name.clone();
                crdts_map.insert(service_name, ServiceCrdt::new(service_def));
            }
        }
        
        for service_def in services {
            let service_name = service_def.name.clone();
            let base_path = service_def.server.base_path.clone();
            registry.register_service(service_def.clone()).await?;
            router.register_service(&service_name, &base_path);
        }

        registry.start_all_services().await?;
        Ok(())
    }
}

