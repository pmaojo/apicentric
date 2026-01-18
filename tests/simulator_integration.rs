<<<<<<< HEAD
use apicentric::adapters::service_spec_loader::YamlServiceSpecLoader;
use apicentric::domain::ports::contract::ServiceSpecLoader;
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig};
=======
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig};
use apicentric::adapters::service_spec_loader::YamlServiceSpecLoader;
use apicentric::domain::ports::contract::ServiceSpecLoader;
>>>>>>> origin/main
use tempfile::TempDir;

#[tokio::test]
async fn simulator_can_load_simple_yaml() {
    // Create a minimal YAML that matches expected structure
    let temp_dir = TempDir::new().unwrap();
    let spec_content = r#"
name: Test Service
version: "1.0"
description: Test service for integration testing
port: 9999
server:
  port: 9999
  base_path: /test
endpoints: []
"#;
<<<<<<< HEAD

    let spec_path = temp_dir.path().join("test-service.yaml");
    std::fs::write(&spec_path, spec_content).unwrap();

    let loader = YamlServiceSpecLoader::new();
    let spec_result = loader.load(spec_path.to_str().unwrap()).await;

=======

    let spec_path = temp_dir.path().join("test-service.yaml");
    std::fs::write(&spec_path, spec_content).unwrap();

    let loader = YamlServiceSpecLoader::new();
    let spec_result = loader.load(spec_path.to_str().unwrap()).await;

>>>>>>> origin/main
    if let Err(ref err) = spec_result {
        println!("Spec load error: {:?}", err);
    }
    assert!(spec_result.is_ok());
    let spec = spec_result.unwrap();
    assert_eq!(spec.name, "Test Service");
    assert_eq!(spec.endpoints.len(), 0);
}

<<<<<<< HEAD
#[tokio::test]
=======
#[tokio::test]
>>>>>>> origin/main
async fn simulator_validates_yaml_structure() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_content = r#"
name: Invalid Service
version: "1.0"
# missing required port field and other structure
endpoints: []
"#;
<<<<<<< HEAD

    let spec_path = temp_dir.path().join("invalid.yaml");
    std::fs::write(&spec_path, invalid_content).unwrap();

    let loader = YamlServiceSpecLoader::new();
    let result = loader.load(spec_path.to_str().unwrap()).await;

=======

    let spec_path = temp_dir.path().join("invalid.yaml");
    std::fs::write(&spec_path, invalid_content).unwrap();

    let loader = YamlServiceSpecLoader::new();
    let result = loader.load(spec_path.to_str().unwrap()).await;

>>>>>>> origin/main
    // Should fail validation due to missing server config
    assert!(result.is_err());
}

#[tokio::test]
async fn simulator_manager_initializes_successfully() {
    let config = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(config);
<<<<<<< HEAD

=======

>>>>>>> origin/main
    // Manager should be created successfully with default config
    assert!(!manager.is_active().await);
}

#[tokio::test]
async fn simulator_can_set_db_path() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
<<<<<<< HEAD

    let config = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(config);

=======

    let config = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(config);

>>>>>>> origin/main
    let result = manager.set_db_path(db_path.to_str().unwrap()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn simulator_status_reflects_state() {
    let config = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(config);
<<<<<<< HEAD

    let status = manager.get_status().await;
    assert_eq!(status.active_services.len(), 0);
    assert!(!status.is_active);
}
=======

    let status = manager.get_status().await;
    assert_eq!(status.active_services.len(), 0);
    assert!(!status.is_active);
}
>>>>>>> origin/main
