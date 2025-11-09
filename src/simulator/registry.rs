//! Service Registry - Manages multiple service instances and their lifecycles

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::{
    config::{PortRange, ServiceDefinition},
    log::RequestLogEntry,
    service::ServiceInstance,
    ServiceInfo,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Port manager for automatic port assignment
pub struct PortManager {
    port_range: PortRange,
    used_ports: Vec<u16>,
}

impl PortManager {
    pub fn new(port_range: PortRange) -> Self {
        Self {
            port_range,
            used_ports: Vec::new(),
        }
    }

    /// Assign a port for a service, either from configuration or automatically
    pub fn assign_port(&mut self, requested_port: Option<u16>) -> ApicentricResult<u16> {
        if let Some(port) = requested_port {
            // Use requested port if available
            if self.used_ports.contains(&port) {
                return Err(ApicentricError::runtime_error(
                    format!("Port {} is already in use", port),
                    Some("Choose a different port or let the system assign one automatically"),
                ));
            }
            self.used_ports.push(port);
            Ok(port)
        } else {
            // Find next available port in range
            for port in self.port_range.start..=self.port_range.end {
                if !self.used_ports.contains(&port) {
                    self.used_ports.push(port);
                    return Ok(port);
                }
            }
            Err(ApicentricError::runtime_error(
                format!(
                    "No available ports in range {}-{}",
                    self.port_range.start, self.port_range.end
                ),
                Some("Increase the port range or stop some services"),
            ))
        }
    }

    /// Release a port when a service stops
    pub fn release_port(&mut self, port: u16) {
        self.used_ports.retain(|&p| p != port);
    }

    /// Get all currently used ports
    pub fn used_ports(&self) -> &[u16] {
        &self.used_ports
    }
}

/// Registry for managing multiple service instances
pub struct ServiceRegistry {
    services: HashMap<String, Arc<RwLock<ServiceInstance>>>,
    port_manager: PortManager,
    storage: Arc<dyn crate::storage::Storage>,
    log_sender: broadcast::Sender<RequestLogEntry>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(
        port_range: PortRange,
        storage: Arc<dyn crate::storage::Storage>,
        log_sender: broadcast::Sender<RequestLogEntry>,
    ) -> Self {
        Self {
            services: HashMap::new(),
            port_manager: PortManager::new(port_range),
            storage,
            log_sender,
        }
    }

    pub fn set_storage(&mut self, storage: Arc<dyn crate::storage::Storage>) {
        self.storage = storage;
    }

    /// Register a new service
    pub async fn register_service(
        &mut self,
        definition: ServiceDefinition,
    ) -> ApicentricResult<()> {
        let service_name = definition.name.clone();

        if self.services.contains_key(&service_name) {
            return Err(ApicentricError::runtime_error(
                format!("Service '{}' is already registered", service_name),
                Some("Use a different service name or unregister the existing service first"),
            ));
        }

        // Assign port for the service
        let port = self.port_manager.assign_port(definition.server.port)?;

        // Create service instance
        let service_instance = ServiceInstance::new(
            definition,
            port,
            self.storage.clone(),
            self.log_sender.clone(),
        )?;

        // Store in registry
        self.services.insert(
            service_name.clone(),
            Arc::new(RwLock::new(service_instance)),
        );

        log::info!("Registered service '{}' on port {}", service_name, port);

        Ok(())
    }

    /// Unregister a service
    pub async fn unregister_service(&mut self, service_name: &str) -> ApicentricResult<()> {
        if let Some(service_arc) = self.services.remove(service_name) {
            let mut service = service_arc.write().await;

            // Stop the service if it's running
            if service.is_running() {
                service.stop().await?;
            }

            // Release the port
            self.port_manager.release_port(service.port());

            log::info!("Unregistered service '{}'", service_name);
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                format!("Service '{}' is not registered", service_name),
                None::<String>,
            ))
        }
    }

    /// Get a service instance by name
    pub fn get_service(&self, service_name: &str) -> Option<&Arc<RwLock<ServiceInstance>>> {
        self.services.get(service_name)
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<ServiceInfo> {
        let mut services = Vec::new();

        for (name, service_arc) in &self.services {
            let service = service_arc.read().await;
            services.push(ServiceInfo {
                name: name.clone(),
                port: service.port(),
                base_path: service.base_path(),
                endpoints_count: service.endpoints_count(),
                is_running: service.is_running(),
            });
        }

        services.sort_by(|a, b| a.name.cmp(&b.name));
        services
    }

    /// Set active scenario for all registered services
    pub async fn set_scenario_all(&self, scenario: Option<String>) {
        for service_arc in self.services.values() {
            let service = service_arc.read().await;
            service.set_scenario(scenario.clone()).await;
        }
    }

    /// Start all registered services
    pub async fn start_all_services(&mut self) -> ApicentricResult<()> {
        let mut errors = Vec::new();

        for (service_name, service_arc) in &self.services {
            let mut service = service_arc.write().await;
            if let Err(e) = service.start().await {
                errors.push(format!("Failed to start service '{}': {}", service_name, e));
            }
        }

        if !errors.is_empty() {
            return Err(ApicentricError::runtime_error(
                format!("Failed to start some services:\n{}", errors.join("\n")),
                Some("Check service configurations and port availability"),
            ));
        }

        log::info!("Started {} services", self.services.len());
        Ok(())
    }

    /// Stop all registered services
    pub async fn stop_all_services(&mut self) -> ApicentricResult<()> {
        let mut errors = Vec::new();

        for (service_name, service_arc) in &self.services {
            let mut service = service_arc.write().await;
            if service.is_running() {
                if let Err(e) = service.stop().await {
                    errors.push(format!("Failed to stop service '{}': {}", service_name, e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(ApicentricError::runtime_error(
                format!("Failed to stop some services:\n{}", errors.join("\n")),
                None::<String>,
            ));
        }

        log::info!("Stopped {} services", self.services.len());
        Ok(())
    }

    /// Get the number of registered services
    pub fn services_count(&self) -> usize {
        self.services.len()
    }

    /// Get the number of running services
    pub async fn running_services_count(&self) -> usize {
        let mut count = 0;
        for service_arc in self.services.values() {
            let service = service_arc.read().await;
            if service.is_running() {
                count += 1;
            }
        }
        count
    }

    /// Check if a service is registered
    pub fn has_service(&self, service_name: &str) -> bool {
        self.services.contains_key(service_name)
    }

    /// Get all service names
    pub fn service_names(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    /// Clear all services (stops and unregisters all)
    pub async fn clear_all_services(&mut self) -> ApicentricResult<()> {
        self.stop_all_services().await?;

        // Clear the services map and reset port manager
        self.services.clear();
        self.port_manager.used_ports.clear();

        log::info!("Cleared all services from registry");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::config::{
        EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig,
    };
    use std::collections::HashMap;

    fn create_test_service_definition(name: &str, port: Option<u16>) -> ServiceDefinition {
        ServiceDefinition {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service".to_string()),
            server: ServerConfig {
                port,
                base_path: format!("/api/{}", name),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            },
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: vec![EndpointDefinition {
                kind: EndpointKind::Http,
                method: "GET".to_string(),
                path: "/test".to_string(),
                header_match: None,
                description: None,
                parameters: None,
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"message": "test"}"#.to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    );
                    responses
                },
                scenarios: None,
                stream: None,
            }],
            graphql: None,
            behavior: None,
        }
    }

    #[test]
    fn test_port_manager_assignment() {
        let mut port_manager = PortManager::new(PortRange {
            start: 8000,
            end: 8002,
        });

        // Test automatic assignment
        let port1 = port_manager.assign_port(None).unwrap();
        assert_eq!(port1, 8000);

        let port2 = port_manager.assign_port(None).unwrap();
        assert_eq!(port2, 8001);

        // Test specific port assignment
        let port3 = port_manager.assign_port(Some(8002)).unwrap();
        assert_eq!(port3, 8002);

        // Test port exhaustion
        let result = port_manager.assign_port(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_port_manager_conflict() {
        let mut port_manager = PortManager::new(PortRange {
            start: 8000,
            end: 8002,
        });

        // Assign a port
        port_manager.assign_port(Some(8001)).unwrap();

        // Try to assign the same port again
        let result = port_manager.assign_port(Some(8001));
        assert!(result.is_err());
    }

    #[test]
    fn test_port_manager_release() {
        let mut port_manager = PortManager::new(PortRange {
            start: 8000,
            end: 8002,
        });

        let port = port_manager.assign_port(Some(8001)).unwrap();
        assert_eq!(port_manager.used_ports().len(), 1);

        port_manager.release_port(port);
        assert_eq!(port_manager.used_ports().len(), 0);

        // Should be able to assign the same port again
        let port2 = port_manager.assign_port(Some(8001)).unwrap();
        assert_eq!(port2, 8001);
    }

    #[tokio::test]
    async fn test_service_registry_registration() {
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut registry = ServiceRegistry::new(
            PortRange {
                start: 9000,
                end: 9999,
            },
            storage,
            tx,
        );

        let service_def = create_test_service_definition("test-service", None);
        let result = registry.register_service(service_def).await;
        assert!(result.is_ok());

        assert_eq!(registry.services_count(), 1);
        assert!(registry.has_service("test-service"));
    }

    #[tokio::test]
    async fn test_service_registry_duplicate_registration() {
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut registry = ServiceRegistry::new(
            PortRange {
                start: 9000,
                end: 9999,
            },
            storage,
            tx,
        );

        let service_def1 = create_test_service_definition("test-service", None);
        let service_def2 = create_test_service_definition("test-service", None);

        registry.register_service(service_def1).await.unwrap();

        let result = registry.register_service(service_def2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_registry_unregistration() {
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut registry = ServiceRegistry::new(
            PortRange {
                start: 9000,
                end: 9999,
            },
            storage,
            tx,
        );

        let service_def = create_test_service_definition("test-service", None);
        registry.register_service(service_def).await.unwrap();

        assert_eq!(registry.services_count(), 1);

        let result = registry.unregister_service("test-service").await;
        assert!(result.is_ok());

        assert_eq!(registry.services_count(), 0);
        assert!(!registry.has_service("test-service"));
    }

    #[tokio::test]
    async fn test_service_registry_list_services() {
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut registry = ServiceRegistry::new(
            PortRange {
                start: 9000,
                end: 9999,
            },
            storage,
            tx,
        );

        let service_def1 = create_test_service_definition("service-a", Some(9001));
        let service_def2 = create_test_service_definition("service-b", Some(9002));

        registry.register_service(service_def1).await.unwrap();
        registry.register_service(service_def2).await.unwrap();

        let services = registry.list_services().await;
        assert_eq!(services.len(), 2);

        // Should be sorted by name
        assert_eq!(services[0].name, "service-a");
        assert_eq!(services[1].name, "service-b");

        assert_eq!(services[0].port, 9001);
        assert_eq!(services[1].port, 9002);
    }
}
