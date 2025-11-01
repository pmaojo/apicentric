use async_trait::async_trait;
use http::{Request, Response};
use mockforge::app::PluginManager;
use mockforge::domain::ports::plugin::Plugin;
use std::fs;
use std::path::PathBuf;

struct TestPlugin;

#[async_trait]
impl Plugin for TestPlugin {
    async fn on_request(&self, request: &mut Request<Vec<u8>>) {
        request
            .headers_mut()
            .insert("x-test", "true".parse().unwrap());
    }

    async fn on_response(&self, response: &mut Response<Vec<u8>>) {
        *response.body_mut() = b"modified".to_vec();
    }
}

#[tokio::test]
async fn plugin_hooks_execute_and_mutate_response() {
    let mut manager = PluginManager::new();
    manager.register_plugin(Box::new(TestPlugin));

    let mut request = Request::new(Vec::new());
    let mut response = Response::new(Vec::new());

    manager.on_request(&mut request).await;
    manager.on_response(&mut response).await;

    assert_eq!(request.headers()["x-test"], "true");
    assert_eq!(response.body(), &b"modified".to_vec());
}

#[tokio::test]
async fn missing_create_plugin_symbol_is_reported() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let mut fake_plugin_path = PathBuf::from(temp_dir.path());
    fake_plugin_path.push(format!(
        "{}missing_symbol.{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_EXTENSION
    ));

    fs::write(&fake_plugin_path, b"not a real plugin").expect("write fake plugin");

    let result = PluginManager::load_from_directory(temp_dir.path());
    assert!(
        result.is_err(),
        "expected error when plugin symbol is missing"
    );

    let error = result.err().unwrap();
    let error_message = error.to_string();
    assert!(
        error_message.contains("Failed to load one or more plugins"),
        "unexpected error message: {}",
        error_message
    );
    assert!(
        error_message.contains("missing_symbol"),
        "expected path in error message: {}",
        error_message
    );
}
