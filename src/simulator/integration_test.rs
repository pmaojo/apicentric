//! Integration tests for the HTTP server and parameter extraction

#[cfg(test)]
mod tests {
    use super::super::config::{
        EndpointDefinition, EndpointKind, ResponseDefinition, ServiceDefinition,
    };
    use super::super::config::{PortRange, SimulatorConfig};
    use super::super::log::RequestLogEntry;
    use super::super::manager::ApiSimulatorManager;
    use super::super::service::ServiceInstance;
    use bytes::Bytes;
    use http_body_util::{BodyExt, Empty};
    use hyper::{Method, Request, StatusCode, Uri};
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use tokio::sync::broadcast;
    use tokio::time::{sleep, Duration};

    use crate::storage::Storage;

    #[derive(Default, Debug)]
    struct RecordingStorage {
        saved: Mutex<Vec<ServiceDefinition>>,
    }

    impl Storage for RecordingStorage {
        fn save_service(&self, service: &ServiceDefinition) -> crate::errors::ApicentricResult<()> {
            let mut guard = self.saved.lock().unwrap();
            guard.push(service.clone());
            Ok(())
        }

        fn load_service(
            &self,
            _name: &str,
        ) -> crate::errors::ApicentricResult<Option<ServiceDefinition>> {
            Ok(None)
        }

        fn append_log(&self, _entry: &RequestLogEntry) -> crate::errors::ApicentricResult<()> {
            Ok(())
        }

        fn query_logs(
            &self,
            _service: Option<&str>,
            _route: Option<&str>,
            _method: Option<&str>,
            _status: Option<u16>,
            _limit: usize,
        ) -> crate::errors::ApicentricResult<Vec<RequestLogEntry>> {
            Ok(Vec::new())
        }
    }

    fn create_test_service_with_params() -> ServiceDefinition {
        ServiceDefinition {
            name: "integration-test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Integration test service".to_string()),
            server: super::super::config::ServerConfig {
                port: Some(8100),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
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
            bucket: None,
            endpoints: vec![
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users".to_string(),
                    header_match: None,
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
                                script: None,
                                headers: None,
                                side_effects: None,
                            },
                        );
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users/{id}".to_string(),
                    header_match: None,
                    description: Some("Get user by ID".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(200, ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"id": "{{ params.id }}", "message": "User {{ params.id }} found"}"#.to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        });
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users/{userId}/orders/{orderId}".to_string(),
                    header_match: None,
                    description: Some("Get user order".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(200, ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: r#"{"userId": "{{ params.userId }}", "orderId": "{{ params.orderId }}", "status": "found"}"#.to_string(),
                            script: None,
                            headers: None,
                            side_effects: None,
                        });
                        responses
                    },
                    scenarios: None,
                    stream: None,
                },
            ],
            graphql: None,
            behavior: None,
        }
    }

    fn create_recording_service() -> ServiceDefinition {
        ServiceDefinition {
            name: "recording-test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Integration test service".to_string()),
            server: super::super::config::ServerConfig {
                port: Some(8200),
                base_path: "/api".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            },
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Vec::new(),
            graphql: None,
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

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, free_port, storage, tx).unwrap();

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
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8101, storage, tx).unwrap();

        // Test single parameter extraction
        let headers = HashMap::new();
        let route_match = service.find_endpoint_with_params("GET", "/users/123", &headers);
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(route_match.path_params.get("id"), Some(&"123".to_string()));

        // Test multiple parameter extraction
        let route_match =
            service.find_endpoint_with_params("GET", "/users/456/orders/789", &headers);
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
        let route_match = service.find_endpoint_with_params("GET", "/products/123", &headers);
        assert!(route_match.is_none());

        // Test wrong method
        let route_match = service.find_endpoint_with_params("POST", "/users/123", &headers);
        assert!(route_match.is_none());
    }

    #[tokio::test]
    async fn test_template_processing_integration() {
        let definition = create_test_service_with_params();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8102, storage, tx).unwrap();

        // Test fixture template processing
        let headers = HashMap::new();
        let route_match = service.find_endpoint_with_params("GET", "/users", &headers);
        assert!(route_match.is_some());

        let _state = service.get_fixtures().await;
        let template = &route_match.unwrap().endpoint.responses[&200].body;

        // The template should contain the fixture placeholder
        assert!(template.contains("{{ fixtures.users }}"));

        // Test parameter template processing
        let route_match = service.find_endpoint_with_params("GET", "/users/123", &headers);
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        let template = &route_match.endpoint.responses[&200].body;

        // The template should contain parameter placeholders
        assert!(template.contains("{{ params.id }}"));

        // Verify parameter extraction worked
        assert_eq!(route_match.path_params.get("id"), Some(&"123".to_string()));
    }

    #[tokio::test]
    async fn test_request_logging_and_retrieval() {
        let definition = create_test_service_with_params();
        let base_path = definition.server.base_path.clone();

        // Use a free port
        let free_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, free_port, storage, tx).unwrap();
        service.start().await.unwrap();
        sleep(Duration::from_millis(100)).await;

        let base_url = format!("http://127.0.0.1:{}{}", free_port, base_path);

        reqwest::get(format!("{}/users", base_url)).await.unwrap();
        reqwest::get(format!("{}/users/1", base_url)).await.unwrap();

        let logs: Vec<RequestLogEntry> =
            reqwest::get(format!("{}/__apicentric/logs?limit=10", base_url))
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

        assert!(logs.len() >= 2);
        assert_eq!(logs[0].path, format!("{}/users", base_path));
        assert_eq!(logs[1].path, format!("{}/users/1", base_path));

        service.stop().await.unwrap();
    }

    fn get_free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }

    #[tokio::test]
    async fn test_unknown_route_recording_creates_endpoint() {
        let mut definition = create_recording_service();
        definition.server.record_unknown = true;

        let port = get_free_port();
        let storage = Arc::new(RecordingStorage::default());
        let (tx, _) = broadcast::channel(10);
        let mut service = ServiceInstance::new(definition, port, storage.clone(), tx).unwrap();

        service.start().await.unwrap();
        sleep(Duration::from_millis(100)).await;

        let client = Client::builder(TokioExecutor::new()).build_http();
        let uri: Uri = format!("http://127.0.0.1:{}/api/orders/42", port)
            .parse()
            .unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let response = client.request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_text.contains("registr"));

        let updated_definition = service.definition();
        let recorded = updated_definition
            .endpoints
            .iter()
            .find(|ep| ep.path == "/orders/{param1}" && ep.method == "GET");
        assert!(recorded.is_some(), "expected recorded endpoint in service");

        let saved = storage.saved.lock().unwrap();
        assert!(
            saved.iter().any(|service| service
                .endpoints
                .iter()
                .any(|ep| ep.path == "/orders/{param1}" && ep.method == "GET")),
            "expected persisted definition with recorded endpoint"
        );

        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_unknown_route_reuse_after_recording() {
        let mut definition = create_recording_service();
        definition.server.record_unknown = true;

        let port = get_free_port();
        let storage = Arc::new(RecordingStorage::default());
        let (tx, _) = broadcast::channel(10);
        let mut service = ServiceInstance::new(definition, port, storage, tx).unwrap();

        service.start().await.unwrap();
        sleep(Duration::from_millis(100)).await;

        let client = Client::builder(TokioExecutor::new()).build_http();
        let uri: Uri = format!("http://127.0.0.1:{}/api/payments/123", port)
            .parse()
            .unwrap();
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri.clone())
            .body(Empty::<Bytes>::new())
            .unwrap();

        let response = client.request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let second_request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let second_response = client.request(second_request).await.unwrap();
        assert_eq!(second_response.status(), StatusCode::OK);
        let body_bytes = second_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json.get("method").and_then(|v| v.as_str()), Some("POST"));
        assert_eq!(
            json.get("path").and_then(|v| v.as_str()),
            Some("/payments/{param1}")
        );

        service.stop().await.unwrap();
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
            db_path: temp_dir.path().join("test.db"),
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
