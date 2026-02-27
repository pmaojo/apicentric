#[cfg(test)]
pub mod tests {
    use crate::errors::ApicentricResult;
    use crate::simulator::config::ServiceDefinition;
    use crate::simulator::config::{
        EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig,
    };
    use crate::simulator::log::RequestLogEntry;
    use crate::simulator::scripting::ScriptingEngine;
    use crate::simulator::service::state::ServiceState;
    use crate::simulator::template::TemplateEngine;
    use crate::storage::{LogStats, Storage};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    struct DummyStorage;

    impl Storage for DummyStorage {
        fn save_service(&self, _service: &ServiceDefinition) -> ApicentricResult<()> {
            Ok(())
        }

        fn load_service(&self, _name: &str) -> ApicentricResult<Option<ServiceDefinition>> {
            Ok(None)
        }

        fn append_log(&self, _entry: &RequestLogEntry) -> ApicentricResult<()> {
            Ok(())
        }

        fn query_logs(
            &self,
            _service: Option<&str>,
            _route: Option<&str>,
            _method: Option<&str>,
            _status: Option<u16>,
            _limit: usize,
        ) -> ApicentricResult<Vec<RequestLogEntry>> {
            Ok(vec![])
        }

        fn get_log_stats(&self) -> ApicentricResult<LogStats> {
            Ok(LogStats::default())
        }

        fn clear_logs(&self) -> ApicentricResult<()> {
            Ok(())
        }
    }

    fn create_test_definition() -> ServiceDefinition {
        ServiceDefinition {
            name: "test-service".to_string(),
            version: Some("1.0.0".to_string()),
            description: None,
            server: Some(ServerConfig {
                port: Some(8080),
                base_path: "/api".to_string(),
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
                path: "/hello".to_string(),
                header_match: None,
                description: None,
                parameters: None,
                request_body: None,
                responses: {
                    let mut map = HashMap::new();
                    map.insert(
                        200,
                        ResponseDefinition {
                            condition: None,
                            content_type: "application/json".to_string(),
                            body: "{\"message\": \"Hello World\"}".to_string(),
                            schema: None,
                            script: None,
                            headers: None,
                            side_effects: None,
                        },
                    );
                    map
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
    async fn test_http_handler_compilation() {
        let def = create_test_definition();
        let _definition = Arc::new(std::sync::RwLock::new(def));
        let storage = Arc::new(DummyStorage);
        let _state = Arc::new(RwLock::new(ServiceState::new(
            None,
            None,
            storage.clone(),
            None,
        )));
        let _template_engine = Arc::new(TemplateEngine::new().unwrap());
        let _scripting_engine = Arc::new(ScriptingEngine::new());
        let _active_scenario = Arc::new(RwLock::new(None::<String>));

        assert!(true);
    }
}
