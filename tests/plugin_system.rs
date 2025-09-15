use async_trait::async_trait;
use http::{Request, Response};
use mockforge::app::PluginManager;
use mockforge::domain::ports::plugin::Plugin;

struct TestPlugin;

#[async_trait]
impl Plugin for TestPlugin {
    async fn on_request(&self, request: &mut Request<Vec<u8>>) {
        request.headers_mut().insert("x-test", "true".parse().unwrap());
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
