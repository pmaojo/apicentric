//! Integration tests for the HTTP server and parameter extraction

#[cfg(test)]
mod tests {
    use super::super::config::{EndpointDefinition, ResponseDefinition, ServiceDefinition};
    use super::super::config::{PortRange, SimulatorConfig};
    use super::super::manager::ApiSimulatorManager;
    use super::super::service::ServiceInstance;
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{Method, Request, Uri};
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    fn create_test_service_with_params() -> ServiceDefinition {
        ServiceDefinition {
            name: "integration-test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Integration test service".to_string()),
            server: super::super::config::ServerConfig {
                port: Some(8100),
                base_path: "/api/v1".to_string(),
                cors: None,
            },
            models: None,
            fixtures: {
                let mut fixtures = HashMap::new();
                fixtures.insert(
                    "users".to_string(),
                    serde_json::json!([
                        {"id": 1, "name": "Alice", "email": "alice@example.com"},
                        {"id": 2, "name": "Bob", "email": "bob@example.com"}
                    ]),
                );
                Some(fixtures)
            },
            endpoints: vec![
                EndpointDefinition {
                    method: "GET".to_string(),
                    path: "/users".to_string(),
                    description: Some("Get all users".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body: "{{ fixtures.users }}".to_string(),
                                headers: None,
                                side_effects: None,
                            },
                        );
                        responses
                    },
                },
                EndpointDefinition {
                    method: "GET".to_string(),
                    path: "/users/{id}".to_string(),
                    description: Some("Get user by ID".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(200, ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"id": "{{ params.id }}", "message": "User {{ params.id }} found"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        });
                        responses
                    },
                },
                EndpointDefinition {
                    method: "GET".to_string(),
                    path: "/users/{userId}/orders/{orderId}".to_string(),
                    description: Some("Get user order".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(200, ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"userId": "{{ params.userId }}", "orderId": "{{ params.orderId }}", "status": "found"}"#.to_string(),
                            headers: None,
                            side_effects: None,
                        });
                        responses
                    },
                },
            ],
            behavior: None,
        }
    }

    #[tokio::test]
    async fn test_http_server_with_parameter_extraction() {
        let definition = create_test_service_with_params();

        // Pick a free ephemeral port to avoid collisions in CI/local runs
        let free_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };

        let mut service = ServiceInstance::new(definition, free_port).unwrap();

        // Start the service
        service.start().await.unwrap();
        assert!(service.is_running());

        // Give the server a moment to start
        sleep(Duration::from_millis(100)).await;

        // Test basic endpoint without parameters
        let client = Client::builder(TokioExecutor::new()).build_http();
        let uri: Uri = format!("http://127.0.0.1:{}/api/v1/users", free_port)
            .parse()
            .unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let response = client.request(request).await;
        assert!(
            response.is_ok(),
            "Failed to make request to /users endpoint"
        );

        // Test endpoint with single parameter
        let uri: Uri = format!("http://127.0.0.1:{}/api/v1/users/123", free_port)
            .parse()
            .unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let response = client.request(request).await;
        assert!(
            response.is_ok(),
            "Failed to make request to /users/123 endpoint"
        );

        // Test endpoint with multiple parameters
        let uri: Uri = format!("http://127.0.0.1:{}/api/v1/users/456/orders/789", free_port)
            .parse()
            .unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let response = client.request(request).await;
        assert!(
            response.is_ok(),
            "Failed to make request to /users/456/orders/789 endpoint"
        );

        // Stop the service
        service.stop().await.unwrap();
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_parameter_extraction_accuracy() {
        let definition = create_test_service_with_params();
        let service = ServiceInstance::new(definition, 8101).unwrap();

        // Test single parameter extraction
        let route_match = service.find_endpoint_with_params("GET", "/users/123");
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(route_match.path_params.get("id"), Some(&"123".to_string()));

        // Test multiple parameter extraction
        let route_match = service.find_endpoint_with_params("GET", "/users/456/orders/789");
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(
            route_match.path_params.get("userId"),
            Some(&"456".to_string())
        );
        assert_eq!(
            route_match.path_params.get("orderId"),
            Some(&"789".to_string())
        );

        // Test non-matching path
        let route_match = service.find_endpoint_with_params("GET", "/products/123");
        assert!(route_match.is_none());

        // Test wrong method
        let route_match = service.find_endpoint_with_params("POST", "/users/123");
        assert!(route_match.is_none());
    }

    #[tokio::test]
    async fn test_template_processing_integration() {
        let definition = create_test_service_with_params();
        let service = ServiceInstance::new(definition, 8102).unwrap();

        // Test fixture template processing
        let route_match = service.find_endpoint_with_params("GET", "/users");
        assert!(route_match.is_some());

        let _state = service.get_fixtures().await;
        let template = &route_match.unwrap().endpoint.responses[&200].body;

        // The template should contain the fixture placeholder
        assert!(template.contains("{{ fixtures.users }}"));

        // Test parameter template processing
        let route_match = service.find_endpoint_with_params("GET", "/users/123");
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        let template = &route_match.endpoint.responses[&200].body;

        // The template should contain parameter placeholders
        assert!(template.contains("{{ params.id }}"));

        // Verify parameter extraction worked
        assert_eq!(route_match.path_params.get("id"), Some(&"123".to_string()));
    }

    fn write_service_file(path: &Path, port: u16) {
        let content = format!(
            "name: test\nserver:\n  port: {port}\n  base_path: /api\nendpoints:\n  - method: GET\n    path: /ping\n    responses:\n      200:\n        content_type: application/json\n        body: '{{{{\"msg\":\"ok\"}}}}'\n",
            port = port
        );
        fs::write(path, content).unwrap();
    }

    #[tokio::test]
    async fn test_automatic_reload_on_yaml_change() {
        let temp_dir = TempDir::new().unwrap();
        let services_dir = temp_dir.path().join("services");
        fs::create_dir_all(&services_dir).unwrap();

        let service_file = services_dir.join("test.yaml");
        write_service_file(&service_file, 9100);

        let config = SimulatorConfig {
            enabled: true,
            services_dir: services_dir.clone(),
            port_range: PortRange {
                start: 9000,
                end: 9200,
            },
            global_behavior: None,
        };

        let manager = ApiSimulatorManager::new(config);
        manager.start().await.unwrap();

        // Allow watcher to start
        sleep(Duration::from_millis(500)).await;

        let status = manager.get_status().await;
        assert_eq!(status.active_services.len(), 1);
        let initial_port = status.active_services[0].port;

        // Modify YAML to change port
        write_service_file(&service_file, initial_port + 1);

        // Give watcher time to detect change and reload
        sleep(Duration::from_secs(1)).await;

        let status = manager.get_status().await;
        assert_eq!(status.active_services[0].port, initial_port + 1);

        manager.stop().await.unwrap();
    }
}
