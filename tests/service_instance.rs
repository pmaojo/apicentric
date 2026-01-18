use std::sync::Arc;

use reqwest::{Client, StatusCode};
use tokio::sync::broadcast;

<<<<<<< HEAD
use apicentric::errors::ApicentricResult;
use apicentric::simulator::config::ServiceDefinition;
use apicentric::simulator::log::RequestLogEntry;
use apicentric::simulator::service::ServiceInstance;
use apicentric::storage::Storage;
=======
use apicentric::simulator::service::ServiceInstance;
use apicentric::simulator::config::ServiceDefinition;
use apicentric::storage::Storage;
use apicentric::errors::ApicentricResult;
use apicentric::simulator::log::RequestLogEntry;
>>>>>>> origin/main

struct NoopStorage;

impl Storage for NoopStorage {
<<<<<<< HEAD
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
    fn clear_logs(&self) -> ApicentricResult<()> {
        Ok(())
    }
=======
    fn save_service(&self, _service: &ServiceDefinition) -> ApicentricResult<()> { Ok(()) }
    fn load_service(&self, _name: &str) -> ApicentricResult<Option<ServiceDefinition>> { Ok(None) }
    fn append_log(&self, _entry: &RequestLogEntry) -> ApicentricResult<()> { Ok(()) }
    fn query_logs(&self, _service: Option<&str>, _route: Option<&str>, _method: Option<&str>, _status: Option<u16>, _limit: usize) -> ApicentricResult<Vec<RequestLogEntry>> { Ok(vec![]) }
>>>>>>> origin/main
}

fn test_service_definition() -> ServiceDefinition {
    let yaml = r#"
name: test
server:
  base_path: /api
  cors:
    enabled: true
    origins: ['*']
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: '{"message": "hi"}'
"#;
    serde_yaml::from_str(yaml).unwrap()
}

#[tokio::test]
async fn get_request_returns_response() {
    let def = test_service_definition();
    let (tx, _) = broadcast::channel(10);
    let storage = Arc::new(NoopStorage);
    let port = 18080;
    let mut service = ServiceInstance::new(def, port, storage, tx).unwrap();
    service.start().await.unwrap();

    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/hello", port);
    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "{\"message\": \"hi\"}");

    service.stop().await.unwrap();
}

#[tokio::test]
async fn cors_preflight_returns_no_content() {
    let def = test_service_definition();
    let (tx, _) = broadcast::channel(10);
    let storage = Arc::new(NoopStorage);
    let port = 18081;
    let mut service = ServiceInstance::new(def, port, storage, tx).unwrap();
    service.start().await.unwrap();

    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/hello", port);
    let resp = client
        .request(reqwest::Method::OPTIONS, &url)
        .header("Origin", "http://example.com")
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    service.stop().await.unwrap();
}

#[tokio::test]
async fn scenario_management_roundtrip() {
    let def = test_service_definition();
    let (tx, _) = broadcast::channel(10);
    let storage = Arc::new(NoopStorage);
    let service = ServiceInstance::new(def, 0, storage, tx).unwrap();

    service.set_scenario(Some("test".to_string())).await;
    assert_eq!(service.get_scenario().await, Some("test".to_string()));
}
<<<<<<< HEAD
=======

>>>>>>> origin/main
