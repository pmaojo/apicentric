use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};

use crate::errors::ApicentricResult;

/// Port trait for HTTP request routing.
#[async_trait]
pub trait RequestRouter: Send + Sync {
    async fn route(&self, req: Request<Full<Bytes>>) -> ApicentricResult<Response<Full<Bytes>>>;
}

/// A very small default router used for tests and as a placeholder. It simply
/// responds with 200 OK for every request.
pub struct DefaultRouter;

#[async_trait]
<<<<<<< HEAD
impl RequestRouter for DefaultRouter {
    async fn route(&self, _req: Request<Full<Bytes>>) -> ApicentricResult<Response<Full<Bytes>>> {
=======
    impl RequestRouter for DefaultRouter {
        async fn route(&self, _req: Request<Full<Bytes>>) -> ApicentricResult<Response<Full<Bytes>>> {
>>>>>>> origin/main
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from_static(b"")))
            .unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_router_returns_ok() {
        let router = DefaultRouter;
<<<<<<< HEAD
        let req = Request::builder()
            .uri("/")
            .body(Full::new(Bytes::new()))
            .unwrap();
=======
        let req = Request::builder().uri("/").body(Full::new(Bytes::new())).unwrap();
>>>>>>> origin/main
        let resp = router.route(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
