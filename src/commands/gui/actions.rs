//! Reusable actions for GUI
//!
//! This module contains shared logic for GUI actions to avoid duplication
//! and ensure consistency.

use super::models::{EndpointInfo, ServiceInfo, ServiceStatus};
use apicentric::simulator::config::ConfigLoader;
use apicentric::simulator::manager::ApiSimulatorManager;
use std::path::PathBuf;

/// Loads services from disk and synchronizes their status with the running simulator manager.
///
/// This function:
/// 1. Reads service definitions from the specified directory using `ConfigLoader`.
/// 2. Converts them into GUI `ServiceInfo` models.
/// 3. Checks the `ApiSimulatorManager`'s registry to determine if services are running.
pub async fn load_and_sync_services(
    manager: &ApiSimulatorManager,
    services_dir: PathBuf,
    default_port: u16,
) -> Result<Vec<ServiceInfo>, String> {
    let config_loader = ConfigLoader::new(services_dir.clone());
    let defs = config_loader
        .load_all_services()
        .map_err(|e| format!("Failed to load services: {}", e))?;

    let mut services = Vec::new();
    for def in defs {
        let name = def.name.clone();
        let path = services_dir.join(format!("{}.yaml", name));
        let port = def
            .server
            .as_ref()
            .and_then(|s| s.port)
            .unwrap_or(default_port);
        let mut info = ServiceInfo::new(name, path, port);
        if let Some(endpoints) = def.endpoints {
            for ep in endpoints {
                info.endpoints.push(EndpointInfo {
                    method: ep.method,
                    path: ep.path,
                });
            }
        }
        info.status = ServiceStatus::Stopped; // Default
        services.push(info);
    }

    // Check which services are actually running
    let registry = manager.service_registry().read().await;
    for svc in &mut services {
        if let Some(inst) = registry.get_service(&svc.name) {
            if inst.read().await.is_running() {
                svc.mark_running();
            }
        }
    }

    Ok(services)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apicentric::simulator::config::{PortRange, SimulatorConfig};
    use std::sync::Arc;

    fn create_test_manager(services_dir: PathBuf) -> ApiSimulatorManager {
        let config = SimulatorConfig {
            enabled: true,
            services_dir,
            port_range: PortRange {
                start: 9000,
                end: 9099,
            },
            db_path: std::path::PathBuf::from(":memory:"),
            admin_port: None,
            global_behavior: None,
        };
        ApiSimulatorManager::new(config)
    }

    #[tokio::test]
    async fn test_load_and_sync_services() {
        let temp = tempfile::tempdir().unwrap();
        let services_dir = temp.path().to_path_buf();

        // Create a dummy service file
        let yaml = r#"
name: test-service
server:
  port: 9001
  base_path: /
endpoints:
  - method: GET
    path: /test
    responses:
      200:
        content_type: application/json
        body: "{}"
"#;
        std::fs::write(services_dir.join("test-service.yaml"), yaml).unwrap();

        let manager = create_test_manager(services_dir.clone());

        // Initial load - should be stopped
        let services = load_and_sync_services(&manager, services_dir.clone(), 8080).await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
        assert_eq!(services[0].status, ServiceStatus::Stopped);
        assert_eq!(services[0].port, 9001);

        // Start the service in manager
        manager.load_services().await.unwrap();
        manager.start_service("test-service").await.unwrap();

        // Sync again - should be running
        let services = load_and_sync_services(&manager, services_dir.clone(), 8080).await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
        assert!(services[0].status.is_running());
    }
}
