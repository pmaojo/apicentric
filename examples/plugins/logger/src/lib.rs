use async_trait::async_trait;
use http::{Request, Response};
use log::info;
use mockforge::domain::ports::plugin::Plugin;

struct LoggerPlugin;

#[async_trait]
impl Plugin for LoggerPlugin {
    async fn on_request(&self, request: &mut Request<Vec<u8>>) {
        info!("-> {} {}", request.method(), request.uri());
    }

    async fn on_response(&self, response: &mut Response<Vec<u8>>) {
        info!("<- {}", response.status());
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(LoggerPlugin)
}
