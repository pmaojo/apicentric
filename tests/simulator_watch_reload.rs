use std::{fs, path::Path, time::Duration};

use mockforge::simulator::{config::PortRange, ApiSimulatorManager, SimulatorConfig};
use tempfile::TempDir;
use tokio::time::sleep;

fn write_service_file(path: &Path, port: u16) {
    let content = format!(
        "name: test\nserver:\n  port: {port}\n  base_path: /api\nendpoints:\n  - method: GET\n    path: /ping\n    responses:\n      200:\n        content_type: application/json\n        body: '{{{{\"msg\":\"ok\"}}}}'\n",
        port = port
    );
    fs::write(path, content).unwrap();
}

#[tokio::test]
async fn watch_reloads_on_yaml_change() {
    let temp_dir = TempDir::new().unwrap();
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&services_dir).unwrap();

    let service_file = services_dir.join("test.yaml");
    write_service_file(&service_file, 9100);

    let config = SimulatorConfig {
        enabled: true,
        services_dir: services_dir.clone(),
        port_range: PortRange { start: 9000, end: 9200 },
        db_path: temp_dir.path().join("test.db"),
        global_behavior: None,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    let status = manager.get_status().await;
    assert_eq!(status.active_services.len(), 1);
    let initial_port = status.active_services[0].port;

    write_service_file(&service_file, initial_port + 1);
    sleep(Duration::from_secs(1)).await;

    let status = manager.get_status().await;
    assert_eq!(status.active_services[0].port, initial_port + 1);

    manager.stop().await.unwrap();
    sleep(Duration::from_millis(500)).await;
}
