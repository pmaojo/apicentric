use async_trait::async_trait;
use http::{Request, Response};

/// Defines hooks for request/response processing that plugins can implement.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Called before a request is processed. Plugins can mutate the request.
    async fn on_request(&self, request: &mut Request<Vec<u8>>);

    /// Called after a response is generated. Plugins can mutate the response.
    async fn on_response(&self, response: &mut Response<Vec<u8>>);
}
