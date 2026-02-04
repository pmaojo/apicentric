#![cfg(feature = "simulator")]

use apicentric::simulator::config::{SimulatorConfig, PortRange};
use apicentric::simulator::manager::ApiSimulatorManager;
use std::env;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[tokio::test]
async fn test_admin_security_scenarios() {
    // 1. Scenario: No Token (Default Secure)
    {
        println!("Running Scenario: No Token");
        let mut temp_dir = env::temp_dir();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        temp_dir.push(format!("apicentric_test_no_token_{}", timestamp));
        fs::create_dir_all(&temp_dir).unwrap();

        let services_dir = temp_dir.join("services");
        fs::create_dir_all(&services_dir).unwrap();

        let service_file = services_dir.join("test_service.yaml");
        fs::write(&service_file, r#"
name: test-service
server:
  base_path: /api
endpoints:
  - path: /test
    method: GET
    responses:
      200:
        content_type: application/json
        body: '{}'
        "#).unwrap();

        let db_path = temp_dir.join("test.db");
        let admin_port = 9099;

        let config = SimulatorConfig {
            enabled: true,
            services_dir,
            port_range: PortRange { start: 9100, end: 9110 },
            db_path,
            admin_port: Some(admin_port),
            global_behavior: None,
        };

        // Ensure token is UNSET
        env::remove_var("APICENTRIC_ADMIN_TOKEN");

        let manager = ApiSimulatorManager::new(config);
        manager.start().await.expect("Failed to start simulator");
        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();
        let url = format!("http://localhost:{}/apicentric-admin/logs", admin_port);

        let resp = client.get(&url).send().await.expect("Failed to send request");
        assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED, "Should be Unauthorized when no token set");

        manager.stop().await.ok();
        fs::remove_dir_all(&temp_dir).ok();
    }

    // 2. Scenario: With Token
    {
        println!("Running Scenario: With Token");
        let mut temp_dir = env::temp_dir();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        temp_dir.push(format!("apicentric_test_with_token_{}", timestamp));
        fs::create_dir_all(&temp_dir).unwrap();

        let services_dir = temp_dir.join("services");
        fs::create_dir_all(&services_dir).unwrap();

        let service_file = services_dir.join("test_service.yaml");
        fs::write(&service_file, r#"
name: test-service
server:
  base_path: /api
endpoints:
  - path: /test
    method: GET
    responses:
      200:
        content_type: application/json
        body: '{}'
        "#).unwrap();

        let db_path = temp_dir.join("test.db");
        let admin_port = 9098;

        let config = SimulatorConfig {
            enabled: true,
            services_dir,
            port_range: PortRange { start: 9111, end: 9120 },
            db_path,
            admin_port: Some(admin_port),
            global_behavior: None,
        };

        let token = "secure-test-token";
        env::set_var("APICENTRIC_ADMIN_TOKEN", token);

        let manager = ApiSimulatorManager::new(config);
        manager.start().await.expect("Failed to start simulator");
        sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();
        let url = format!("http://localhost:{}/apicentric-admin/logs", admin_port);

        // Valid token
        let resp = client.get(&url).header("Authorization", format!("Bearer {}", token)).send().await.expect("Failed");
        assert_eq!(resp.status(), reqwest::StatusCode::OK, "Should be OK with valid token");

        // Invalid token
        let resp_invalid = client.get(&url).header("Authorization", "Bearer wrong").send().await.expect("Failed");
        assert_eq!(resp_invalid.status(), reqwest::StatusCode::FORBIDDEN, "Should be Forbidden with invalid token");

        manager.stop().await.ok();
        fs::remove_dir_all(&temp_dir).ok();
        env::remove_var("APICENTRIC_ADMIN_TOKEN");
    }
}
