use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};

use crate::errors::PulseResult;

/// Port trait for HTTP request routing.
#[async_trait]
pub trait RequestRouter: Send + Sync {
    async fn route(&self, req: Request<Full<Bytes>>) -> PulseResult<Response<Full<Bytes>>>;
}

/// A very small default router used for tests and as a placeholder. It simply
/// responds with 200 OK for every request.
pub struct DefaultRouter;

#[async_trait]
    impl RequestRouter for DefaultRouter {
        async fn route(&self, _req: Request<Full<Bytes>>) -> PulseResult<Response<Full<Bytes>>> {
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
        let req = Request::builder().uri("/").body(Full::new(Bytes::new())).unwrap();
        let resp = router.route(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
