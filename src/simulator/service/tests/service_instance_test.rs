#[cfg(test)]
pub mod tests {
    use crate::simulator::config::{
        EndpointDefinition, EndpointKind, ResponseDefinition, ScenarioConditions,
        ScenarioDefinition, ScenarioResponse, ScenarioStrategy, ServerConfig, ServiceDefinition,
        SideEffect,
    };
    use crate::simulator::service::state::ServiceState;
    use crate::simulator::service::{http_handler, ServiceInstance};
    use crate::simulator::template::{RequestContext, TemplateContext, TemplateEngine};
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full};
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{
        Request as HyperRequest, Response as HyperResponse, StatusCode as HyperStatusCode,
    };
    use hyper_util::rt::TokioIo;
    use reqwest::StatusCode as ReqStatusCode;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::sync::{Arc, RwLock};
    use tokio::net::TcpListener;
    use tokio::sync::{broadcast, RwLock as TokioRwLock};
    use tokio::time::{sleep, Duration};

    fn create_test_service_definition() -> ServiceDefinition {
        ServiceDefinition {
            name: "test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service".to_string()),
            server: Some(ServerConfig {
                port: Some(8001),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: {
                let mut fixtures = HashMap::new();
                fixtures.insert(
                    "users".to_string(),
                    serde_json::json!([
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]),
                );
                Some(fixtures)
            },
            bucket: None,
            endpoints: Some(vec![
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
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/users/1".to_string(),
                    header_match: None,
                    description: Some("Get user by ID".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body: r#"{"id": 1, "name": "Alice"}"#.to_string(),
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
                },
            ]),
            graphql: None,
            behavior: None,
            #[cfg(feature = "iot")]
            twin: None,
        }
    }

    #[tokio::test]
    async fn test_service_instance_creation() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8003, storage, tx).unwrap(); // Use different port

        assert_eq!(service.name(), "test-service");
        assert_eq!(service.port(), 8003);
        assert_eq!(service.base_path(), "/api/v1");
        assert_eq!(service.endpoints_count(), 2);
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_service_start_stop() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, 8002, storage, tx).unwrap(); // Use different port to avoid conflicts

        assert!(!service.is_running());

        service.start().await.unwrap();
        assert!(service.is_running());

        service.stop().await.unwrap();
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_service_state_management() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8004, storage, tx).unwrap(); // Use different port

        // Test fixture access
        let users = service.get_state("users").await.unwrap();
        assert!(users.is_array());

        // Test runtime data
        service
            .update_state("test_key", serde_json::json!("test_value"))
            .await;
        let value = service.get_state("test_key").await.unwrap();
        assert_eq!(value, serde_json::json!("test_value"));
    }

    #[tokio::test]
    async fn test_fixture_array_operations() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8010, storage, tx).unwrap();

        // Test adding to fixture array
        let new_user = serde_json::json!({"id": 3, "name": "Charlie"});
        service
            .add_to_fixture_array("users", new_user)
            .await
            .unwrap();

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array.len(), 3);
        assert_eq!(users_array[2]["name"], "Charlie");

        // Test updating array item by index
        let updated_user = serde_json::json!({"id": 3, "name": "Charles"});
        service
            .update_fixture_array_item("users", 2, updated_user)
            .await
            .unwrap();

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array[2]["name"], "Charles");

        // Test removing from array by index
        let removed_user = service.remove_from_fixture_array("users", 1).await.unwrap();
        assert_eq!(removed_user["name"], "Bob");

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array.len(), 2);
    }

    #[tokio::test]
    async fn test_fixture_array_operations_by_field() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8011, storage, tx).unwrap();

        // Test updating by field value
        let updated_user = serde_json::json!({"id": 1, "name": "Alice Updated"});
        let found = service
            .update_fixture_array_item_by_field("users", "id", &serde_json::json!(1), updated_user)
            .await
            .unwrap();
        assert!(found);

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array[0]["name"], "Alice Updated");

        // Test removing by field value
        let removed_user = service
            .remove_fixture_array_item_by_field("users", "id", &serde_json::json!(2))
            .await
            .unwrap();
        assert!(removed_user.is_some());
        assert_eq!(removed_user.unwrap()["name"], "Bob");

        let users = service.get_fixtures().await;
        let users_array = users.get("users").unwrap().as_array().unwrap();
        assert_eq!(users_array.len(), 1);
    }

    #[tokio::test]
    async fn test_fixture_reset() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8012, storage, tx).unwrap();

        // Modify fixtures
        service
            .add_to_fixture_array("users", serde_json::json!({"id": 3, "name": "Charlie"}))
            .await
            .unwrap();
        service
            .update_fixture("new_fixture", serde_json::json!("test"))
            .await;

        // Verify modifications
        let users = service.get_fixtures().await;
        assert_eq!(users.get("users").unwrap().as_array().unwrap().len(), 3);
        assert!(users.contains_key("new_fixture"));

        // Reset fixtures
        service.reset_fixtures().await;

        // Verify reset
        let users = service.get_fixtures().await;
        assert_eq!(users.get("users").unwrap().as_array().unwrap().len(), 2);
        assert!(!users.contains_key("new_fixture"));
    }

    #[tokio::test]
    async fn test_runtime_data_management() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8013, storage, tx).unwrap();

        // Test setting runtime data
        service
            .set_runtime_data("session_id", serde_json::json!("abc123"))
            .await;
        service
            .set_runtime_data("user_count", serde_json::json!(42))
            .await;

        // Test getting runtime data
        let session_id = service.get_runtime_data("session_id").await.unwrap();
        assert_eq!(session_id, serde_json::json!("abc123"));

        // Test checking existence
        assert!(service.has_runtime_data("session_id").await);
        assert!(!service.has_runtime_data("nonexistent").await);

        // Test removing runtime data
        let removed = service.remove_runtime_data("session_id").await.unwrap();
        assert_eq!(removed, serde_json::json!("abc123"));
        assert!(!service.has_runtime_data("session_id").await);

        // Test clearing all runtime data
        service.clear_runtime_data().await;
        assert!(!service.has_runtime_data("user_count").await);

        let (fixture_count, runtime_count) = service.get_state_info().await;
        assert_eq!(fixture_count, 1); // users fixture
        assert_eq!(runtime_count, 0);
    }

    #[tokio::test]
    async fn test_side_effects_processing() {
        use crate::simulator::service::routing::PathParameters;

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let mut state = ServiceState::new(
            Some({
                let mut fixtures = HashMap::new();
                fixtures.insert("users".to_string(), serde_json::json!([]));
                fixtures
            }),
            None,
            storage,
            None,
        );

        let template_engine = TemplateEngine::new().unwrap();
        let params = PathParameters::new();
        let request_context = RequestContext::from_request_data(
            "POST".to_string(),
            "/users".to_string(),
            HashMap::new(),
            HashMap::new(),
            Some(serde_json::json!({"id": 1, "name": "Alice"})),
        );
        let template_context = TemplateContext::new(&state, &params, request_context);

        // Test add_to_fixture side effect
        let side_effect = SideEffect {
            action: "add_to_fixture".to_string(),
            target: "users".to_string(),
            value: r#"{"id": 1, "name": "Alice"}"#.to_string(),
        };

        crate::simulator::service::response_processor::process_side_effect(
            &side_effect,
            &mut state,
            &template_context,
            &template_engine,
        )
        .unwrap();

        let users = state.get_fixture("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["name"], "Alice");

        // Test set_runtime_data side effect
        let side_effect = SideEffect {
            action: "set_runtime_data".to_string(),
            target: "last_user_id".to_string(),
            value: "1".to_string(),
        };

        crate::simulator::service::response_processor::process_side_effect(
            &side_effect,
            &mut state,
            &template_context,
            &template_engine,
        )
        .unwrap();

        let last_id = state.get_runtime_data("last_user_id").unwrap();
        assert_eq!(last_id, &serde_json::json!(1));
    }

    #[tokio::test]
    async fn test_endpoint_finding() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8005, storage, tx).unwrap(); // Use different port

        let headers = HashMap::new();
        let endpoint = service.find_endpoint("GET", "/users", &headers);
        assert!(endpoint.is_some());
        assert_eq!(endpoint.unwrap().path, "/users");

        let endpoint = service.find_endpoint("POST", "/users", &headers);
        assert!(endpoint.is_none());

        let endpoint = service.find_endpoint("get", "/users", &headers); // Case insensitive
        assert!(endpoint.is_some());
    }

    #[tokio::test]
    async fn test_path_parameter_extraction() {
        let definition = create_test_service_definition_with_params();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8008, storage, tx).unwrap();

        // Test parameter extraction
        let headers = HashMap::new();
        let route_match = service.find_endpoint_with_params("GET", "/users/123", &headers);
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(route_match.endpoint.path, "/users/{id}");
        assert_eq!(route_match.path_params.get("id"), Some(&"123".to_string()));

        // Test multiple parameters
        let route_match =
            service.find_endpoint_with_params("GET", "/users/123/orders/456", &headers);
        assert!(route_match.is_some());

        let route_match = route_match.unwrap();
        assert_eq!(
            route_match.path_params.get("userId"),
            Some(&"123".to_string())
        );
        assert_eq!(
            route_match.path_params.get("orderId"),
            Some(&"456".to_string())
        );

        // Test no match
        let route_match = service.find_endpoint_with_params("GET", "/products/123", &headers);
        assert!(route_match.is_none());
    }

    #[tokio::test]
    async fn test_endpoint_header_matching() {
        let definition = create_test_service_definition_with_header_match();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8010, storage, tx).unwrap();

        let mut headers = HashMap::new();
        // Without specific header, it should match the fallback
        let endpoint = service.find_endpoint("GET", "/headers", &headers);
        assert!(endpoint.is_some());
        assert_eq!(
            endpoint.unwrap().description,
            Some("Get without header match".to_string())
        );

        // Correct header should match the restricted endpoint
        headers.insert("x-test".to_string(), "true".to_string());
        let endpoint = service.find_endpoint("GET", "/headers", &headers);
        assert!(endpoint.is_some());
        assert_eq!(
            endpoint.unwrap().description,
            Some("Get with header match".to_string())
        );
    }

    #[test]
    fn test_endpoint_path_to_regex() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8009, storage, tx).unwrap();

        // Test simple parameter
        let regex = service.endpoint_path_to_regex("/users/{id}");
        assert_eq!(regex, "^/users/(?P<id>[^/]+)$");

        // Test multiple parameters
        let regex = service.endpoint_path_to_regex("/users/{userId}/orders/{orderId}");
        assert_eq!(
            regex,
            "^/users/(?P<userId>[^/]+)/orders/(?P<orderId>[^/]+)$"
        );

        // Test no parameters
        let regex = service.endpoint_path_to_regex("/users");
        assert_eq!(regex, "^/users$");
    }

    #[tokio::test]
    async fn test_template_processing_with_params() {
        use crate::simulator::service::routing::PathParameters;

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let mut state = ServiceState::new(None, None, storage, None);
        state.set_fixture(
            "users".to_string(),
            serde_json::json!([{"id": 1, "name": "Alice"}]),
        );

        let mut params = PathParameters::new();
        params.insert("id".to_string(), "123".to_string());

        let request_context = RequestContext::from_request_data(
            "GET".to_string(),
            "/users/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        );

        let template_context = TemplateContext::new(&state, &params, request_context);
        let engine = TemplateEngine::new().unwrap();

        let template = r#"{"user_id": "{{params.id}}", "users": {{json fixtures.users}}}"#;
        let result = engine.render(template, &template_context);

        // Debug print to see what we got

        let result = result.unwrap();
        assert!(result.contains(r#""user_id": "123""#));
    }

    fn create_test_service_definition_with_params() -> ServiceDefinition {
        ServiceDefinition {
            name: "test-service-params".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service with parameters".to_string()),
            server: Some(ServerConfig {
                port: Some(8001),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: {
                let mut fixtures = HashMap::new();
                fixtures.insert(
                    "users".to_string(),
                    serde_json::json!([
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]),
                );
                Some(fixtures)
            },
            bucket: None,
            endpoints: Some(vec![
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
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body:
                                    r#"{"id": "{{ params.id }}", "name": "User {{ params.id }}"}"#
                                        .to_string(),
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
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body:
                                    r#"{"userId": "{{ params.userId }}", "orderId": "{{ params.orderId }}", "status": "found"}"#
                                        .to_string(),
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
                },
            ]),
            graphql: None,
            behavior: None,
            #[cfg(feature = "iot")]
            twin: None,
        }
    }

    fn create_test_service_definition_with_header_match() -> ServiceDefinition {
        ServiceDefinition {
            name: "test-service-headers".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test service with header match".to_string()),
            server: Some(ServerConfig {
                port: Some(8001),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Some(vec![
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/headers".to_string(),
                    header_match: Some({
                        let mut headers = HashMap::new();
                        headers.insert("X-Test".to_string(), "true".to_string());
                        headers
                    }),
                    description: Some("Get with header match".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body: r#"{"msg": "header matched"}"#.to_string(),
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
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".to_string(),
                    path: "/headers".to_string(),
                    header_match: None,
                    description: Some("Get without header match".to_string()),
                    parameters: None,
                    request_body: None,
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert(
                            200,
                            ResponseDefinition {
                                condition: None,
                                content_type: "application/json".to_string(),
                                body: r#"{"msg": "default response"}"#.to_string(),
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
                },
            ]),
            graphql: None,
            behavior: None,
            #[cfg(feature = "iot")]
            twin: None,
        }
    }

    #[tokio::test]
    async fn test_service_validation() {
        let definition = create_test_service_definition();
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8006, storage, tx).unwrap(); // Use different port

        let result = service.validate_consistency();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_duplicate_endpoint_validation() {
        let mut definition = create_test_service_definition();

        // Add duplicate endpoint
        definition
            .endpoints
            .as_mut()
            .unwrap()
            .push(EndpointDefinition {
                kind: EndpointKind::Http,
                method: "GET".to_string(),
                path: "/users".to_string(), // Duplicate path with same method
                description: Some("Duplicate endpoint".to_string()),
                header_match: None,
                parameters: None,
                request_body: None,
                responses: HashMap::new(),
                scenarios: None,
                stream: None,
            });

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let service = ServiceInstance::new(definition, 8007, storage, tx).unwrap(); // Use different port
        let result = service.validate_consistency();
        assert!(result.is_err());
    }

    #[test]
    fn test_endpoint_path_to_regex_handles_unclosed_brace() {
        let pattern = ServiceInstance::endpoint_path_to_regex_static("/users/{id");
        assert!(pattern.contains("users"));
    }

    #[tokio::test]
    async fn test_scenario_matching() {
        // Build endpoint with various scenarios
        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/test".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: Some(vec![
                ScenarioDefinition {
                    name: Some("query".to_string()),
                    conditions: Some(ScenarioConditions {
                        query: Some(HashMap::from([("mode".to_string(), "1".to_string())])),
                        headers: None,
                        body: None,
                    }),
                    response: ScenarioResponse {
                        status: 200,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"result\":\"query\"}".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
                ScenarioDefinition {
                    name: Some("header".to_string()),
                    conditions: Some(ScenarioConditions {
                        query: None,
                        headers: Some(HashMap::from([("x-scn".to_string(), "hdr".to_string())])),
                        body: None,
                    }),
                    response: ScenarioResponse {
                        status: 201,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"result\":\"header\"}".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
                ScenarioDefinition {
                    name: Some("body".to_string()),
                    conditions: Some(ScenarioConditions {
                        query: None,
                        headers: None,
                        body: Some(HashMap::from([(
                            "kind".to_string(),
                            serde_json::json!("b"),
                        )])),
                    }),
                    response: ScenarioResponse {
                        status: 202,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"result\":\"body\"}".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
                ScenarioDefinition {
                    name: Some("error".to_string()),
                    conditions: None,
                    response: ScenarioResponse {
                        status: 500,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"error\":\"forced\"}".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                    strategy: None,
                },
            ]),
            stream: None,
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let state = Arc::new(TokioRwLock::new(ServiceState::new(
            None, None, storage, None,
        )));

        // Query condition
        let mut query = HashMap::new();
        query.insert("mode".to_string(), "1".to_string());
        let res = crate::simulator::service::scenario_matcher::match_scenario(
            &endpoint,
            &state,
            0,
            None,
            &query,
            &HashMap::new(),
            &None,
        )
        .await;
        assert_eq!(res.unwrap().0, 200);

        // Header condition
        let mut headers = HashMap::new();
        headers.insert("x-scn".to_string(), "hdr".to_string());
        let res = crate::simulator::service::scenario_matcher::match_scenario(
            &endpoint,
            &state,
            0,
            None,
            &HashMap::new(),
            &headers,
            &None,
        )
        .await;
        assert_eq!(res.unwrap().0, 201);

        // Body condition
        let body = Some(serde_json::json!({"kind": "b"}));
        let res = crate::simulator::service::scenario_matcher::match_scenario(
            &endpoint,
            &state,
            0,
            None,
            &HashMap::new(),
            &HashMap::new(),
            &body,
        )
        .await;
        assert_eq!(res.unwrap().0, 202);

        // Active scenario fallback
        let res = crate::simulator::service::scenario_matcher::match_scenario(
            &endpoint,
            &state,
            0,
            Some("error".to_string()),
            &HashMap::new(),
            &HashMap::new(),
            &None,
        )
        .await;
        assert_eq!(res.unwrap().0, 500);
    }

    #[tokio::test]
    async fn test_scenario_rotation_sequential() {
        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/rotate".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: Some(vec![
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Sequential),
                    response: ScenarioResponse {
                        status: 200,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "a".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Sequential),
                    response: ScenarioResponse {
                        status: 201,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "b".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Sequential),
                    response: ScenarioResponse {
                        status: 202,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "c".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
            ]),
            stream: None,
        };
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let state = Arc::new(TokioRwLock::new(ServiceState::new(
            None, None, storage, None,
        )));
        let mut results = Vec::new();
        for _ in 0..4 {
            let res = crate::simulator::service::scenario_matcher::match_scenario(
                &endpoint,
                &state,
                0,
                None,
                &HashMap::new(),
                &HashMap::new(),
                &None,
            )
            .await
            .unwrap()
            .0;
            results.push(res);
        }
        assert_eq!(results, vec![200, 201, 202, 200]);
    }

    #[tokio::test]
    async fn test_scenario_rotation_random() {
        use std::collections::HashSet;

        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/random".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: HashMap::new(),
            scenarios: Some(vec![
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Random),
                    response: ScenarioResponse {
                        status: 200,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "ok".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
                ScenarioDefinition {
                    name: None,
                    conditions: None,
                    strategy: Some(ScenarioStrategy::Random),
                    response: ScenarioResponse {
                        status: 500,
                        definition: ResponseDefinition {
                            condition: None,
                            content_type: "text/plain".to_string(),
                            body: "err".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    },
                },
            ]),
            stream: None,
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let state = Arc::new(TokioRwLock::new(ServiceState::new(
            None, None, storage, None,
        )));
        let mut statuses = HashSet::new();
        for _ in 0..20 {
            let res = crate::simulator::service::scenario_matcher::match_scenario(
                &endpoint,
                &state,
                0,
                None,
                &HashMap::new(),
                &HashMap::new(),
                &None,
            )
            .await
            .unwrap()
            .0;
            statuses.insert(res);
        }
        assert_eq!(statuses.len(), 2);
    }

    fn spawn_upstream_server(port: u16) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
            if let Ok((stream, _)) = listener.accept().await {
                let io = TokioIo::new(stream);
                let service = service_fn(|req: HyperRequest<hyper::body::Incoming>| async move {
                    let (parts, body) = req.into_parts();
                    let header_val = parts
                        .headers
                        .get("x-test-header")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");
                    let bytes = BodyExt::collect(body).await.unwrap().to_bytes();
                    let body_str = String::from_utf8_lossy(&bytes);
                    let resp_body = format!("header={};body={}", header_val, body_str);
                    Ok::<_, Infallible>(
                        HyperResponse::builder()
                            .status(HyperStatusCode::OK)
                            .header("x-test-header", header_val)
                            .body(Full::new(Bytes::from(resp_body)))
                            .unwrap(),
                    )
                });
                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Upstream server error: {}", e);
                }
            }
        })
    }

    #[tokio::test]
    async fn test_proxy_forwards_unmatched_requests() {
        let upstream_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };
        let upstream_handle = spawn_upstream_server(upstream_port);

        let mut definition = create_test_service_definition();
        definition.server.as_mut().unwrap().proxy_base_url =
            Some(format!("http://127.0.0.1:{}", upstream_port));

        let service_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, service_port, storage, tx).unwrap();
        service.start().await.unwrap();
        sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/api/v1/unknown", service_port);
        let resp = client
            .post(&url)
            .header("x-test-header", "abc")
            .body("hello")
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), ReqStatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("x-test-header")
                .and_then(|v| v.to_str().ok()),
            Some("abc")
        );
        let body = resp.text().await.unwrap();
        assert_eq!(body, "header=abc;body=hello");

        service.stop().await.unwrap();
        upstream_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_proxy_disabled_returns_not_found() {
        let mut definition = create_test_service_definition();
        definition.server.as_mut().unwrap().proxy_base_url = None; // ensure proxy disabled

        let service_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            port
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(100);
        let mut service = ServiceInstance::new(definition, service_port, storage, tx).unwrap();
        service.start().await.unwrap();
        sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/api/v1/unknown", service_port);
        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), ReqStatusCode::NOT_FOUND);

        service.stop().await.unwrap();
    }

    #[test]
    fn test_service_state_operations() {
        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let mut state = ServiceState::new(None, None, storage, None);

        // Test runtime data
        state.set_runtime_data("key1".to_string(), serde_json::json!("value1"));
        assert_eq!(
            state.get_runtime_data("key1"),
            Some(&serde_json::json!("value1"))
        );

        // Test fixtures
        state.set_fixture("fixture1".to_string(), serde_json::json!({"data": "test"}));
        assert_eq!(
            state.get_fixture("fixture1"),
            Some(&serde_json::json!({"data": "test"}))
        );

        // Test non-existent keys
        assert_eq!(state.get_runtime_data("nonexistent"), None);
        assert_eq!(state.get_fixture("nonexistent"), None);

        // Test data bucket
        let bucket = state.bucket();
        bucket.set("foo".to_string(), serde_json::json!(123));
        assert_eq!(bucket.get("foo"), Some(serde_json::json!(123)));
    }
}
