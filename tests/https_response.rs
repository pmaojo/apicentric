use std::collections::HashMap;
use std::sync::Arc;
use rcgen::generate_simple_self_signed;
use tempfile::tempdir;
use tokio::sync::broadcast;
use mockforge::simulator::config::{ServiceDefinition, ServerConfig, EndpointDefinition, EndpointKind, ResponseDefinition};
use mockforge::simulator::service::ServiceInstance;
use mockforge::storage::sqlite::SqliteStorage;

#[tokio::test]
async fn https_service_responds() {
    let cert = generate_simple_self_signed(["localhost".into()].to_vec()).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();
    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("cert.pem");
    let key_path = dir.path().join("key.pem");
    std::fs::write(&cert_path, cert_pem).unwrap();
    std::fs::write(&key_path, key_pem).unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let response = ResponseDefinition {
        condition: None,
        content_type: "text/plain".into(),
        body: "hello".into(),
        script: None,
        headers: None,
        side_effects: None,
    };
    let mut responses = HashMap::new();
    responses.insert(200, response);

    let endpoint = EndpointDefinition {
        kind: EndpointKind::Http,
        method: "GET".into(),
        path: "/hello".into(),
        header_match: None,
        description: None,
        parameters: None,
        request_body: None,
        responses,
        scenarios: None,
        stream: None,
    };

    let service_def = ServiceDefinition {
        name: "tls".into(),
        version: None,
        description: None,
        server: ServerConfig {
            port: Some(port),
            base_path: "/".into(),
            proxy_base_url: None,
            cors: None,
            cert: Some(cert_path.clone()),
            key: Some(key_path.clone()),
        },
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: vec![endpoint],
        graphql: None,
        behavior: None,
    };

    let db_path = dir.path().join("db.sqlite");
    let storage = Arc::new(SqliteStorage::init_db(db_path).unwrap());
    let (tx, _rx) = broadcast::channel(1);
    let mut service = ServiceInstance::new(service_def, port, storage, tx).unwrap();
    service.start().await.unwrap();

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = client
        .get(format!("https://localhost:{}/hello", port))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "hello");

    service.stop().await.unwrap();
}
