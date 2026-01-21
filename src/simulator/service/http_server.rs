use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;

use super::router::RequestRouter;
use crate::errors::{ApicentricError, ApicentricResult};

/// Adapter responsible for running an HTTP server that delegates
/// requests to a [`RequestRouter`].
pub struct HttpServer<R: RequestRouter + 'static> {
    addr: SocketAddr,
    router: Arc<R>,
    handle: Option<JoinHandle<()>>,
    listener_addr: Option<SocketAddr>,
}

impl<R: RequestRouter + 'static> HttpServer<R> {
    pub fn new(addr: SocketAddr, router: Arc<R>) -> Self {
        Self {
            addr,
            router,
            handle: None,
            listener_addr: None,
        }
    }

    /// Start listening for HTTP requests.
    pub async fn start(&mut self) -> ApicentricResult<()> {
        if self.handle.is_some() {
            return Err(ApicentricError::runtime_error(
                "server already running",
                None::<String>,
            ));
        }
        let listener = TcpListener::bind(self.addr).await.map_err(|e| {
            ApicentricError::runtime_error(format!("failed to bind: {}", e), None::<String>)
        })?;
        self.listener_addr = Some(listener.local_addr().unwrap());
        let router = Arc::clone(&self.router);
        self.handle = Some(tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let router = Arc::clone(&router);
                tokio::spawn(async move {
                    let svc = service_fn(move |req: Request<hyper::body::Incoming>| {
                        let router = Arc::clone(&router);
                        async move {
                            let (parts, body) = req.into_parts();
                            use http_body_util::BodyExt;
                            let bytes = match body.collect().await {
                                Ok(collected) => collected.to_bytes(),
                                Err(_) => Bytes::new(),
                            };
                            let req = Request::from_parts(parts, Full::new(bytes));
                            match router.route(req).await {
                                Ok(resp) => Ok::<_, Infallible>(resp),
                                Err(e) => {
                                    eprintln!("router error: {}", e);
                                    Ok(Response::builder()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .body(Full::new(Bytes::new()))
                                        .unwrap())
                                }
                            }
                        }
                    });
                    let _ = http1::Builder::new()
                        .serve_connection(TokioIo::new(stream), svc)
                        .await;
                });
            }
        }));
        Ok(())
    }

    /// Stop the server if it is running.
    pub async fn stop(&mut self) -> ApicentricResult<()> {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        Ok(())
    }

    /// Return the address the server is listening on.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.listener_addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    struct TestRouter;
    #[async_trait::async_trait]
    impl RequestRouter for TestRouter {
        async fn route(
            &self,
            _req: Request<Full<Bytes>>,
        ) -> ApicentricResult<Response<Full<Bytes>>> {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
    }

    #[tokio::test]
    async fn server_handles_request() {
        let router = Arc::new(TestRouter);
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let mut server = HttpServer::new(addr, router);
        server.start().await.unwrap();
        let addr = server.local_addr().unwrap();

        let client = Client::new();
        let resp = client.get(format!("http://{addr}/")).send().await.unwrap();
        assert_eq!(resp.status(), reqwest::StatusCode::OK);

        server.stop().await.unwrap();
    }
}
