use apicentric::simulator::{
    config::{
        EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
        SimulatorConfig,
    },
    manager::ApiSimulatorManager,
};
use std::collections::HashMap;
use tokio::sync::OnceCell;

static SIMULATOR_MANAGER: OnceCell<ApiSimulatorManager> = OnceCell::const_new();

async fn global_simulator_manager() -> &'static ApiSimulatorManager {
    SIMULATOR_MANAGER
        .get_or_init(|| async {
            let services_dir = tempfile::tempdir().unwrap();
            let service_def = create_test_service_definition("test-service", Some(9001));
            let service_path = services_dir.path().join("test-service.yaml");
            serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def)
                .unwrap();

            let mut config = SimulatorConfig::default_config();
            config.enabled = true;
            config.admin_port = Some(9999);
            config.services_dir = services_dir.path().to_path_buf();

            let manager = ApiSimulatorManager::new(config);
            manager.start().await.unwrap();
            manager
        })
        .await
}

fn create_test_service_definition(name: &str, port: Option<u16>) -> ServiceDefinition {
    ServiceDefinition {
        name: name.to_string(),
        version: Some("1.0.0".to_string()),
        description: Some("Test service".to_string()),
        server: Some(ServerConfig {
            port,
            base_path: format!("/api/{}", name),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        }),
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Some(vec![EndpointDefinition {
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
                        schema: None,
                        script: None,
                        headers: None,
                        side_effects: None,
                    },
                );
                responses
            },
            scenarios: None,
            stream: None,
        }]),
        graphql: None,
        behavior: None,
        #[cfg(feature = "iot")]
        twin: None,
    }
}

#[tokio::test]
async fn test_admin_server_log_retrieval() {
    global_simulator_manager().await;

    // Make a request to the service to generate a log entry
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:9001/api/test-service/test")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // Give time for the log to be recorded and broadcast
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Set the admin token
    std::env::set_var("APICENTRIC_ADMIN_TOKEN", "test-token");

    // Make a request to the admin API to get the logs
    let res = client
        .get("http://localhost:9999/apicentric-admin/logs")
        .bearer_auth("test-token")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let logs: Vec<apicentric::simulator::log::RequestLogEntry> = res.json().await.unwrap();
    assert!(!logs.is_empty());
    let log_entry = logs
        .iter()
        .find(|l| l.path == "/api/test-service/test")
        .unwrap();
    assert_eq!(log_entry.service, "test-service");
}
