//! Admin server for the API simulator.
use crate::simulator::registry::ServiceRegistry;
use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub struct AdminServer {
    service_registry: Arc<RwLock<ServiceRegistry>>,
    server_handle: Option<JoinHandle<()>>,
}

impl AdminServer {
    pub fn new(service_registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        Self {
            service_registry,
            server_handle: None,
        }
    }

    pub async fn start(&mut self, port: u16) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await.unwrap();
        let service_registry = self.service_registry.clone();

        let server_handle = tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                let service_registry = service_registry.clone();

                tokio::task::spawn(async move {
                    let service = service_fn(move |req| {
                        let service_registry = service_registry.clone();
                        async move {
<<<<<<< HEAD
                            Ok::<_, Infallible>(handle_admin_request(req, service_registry).await)
=======
                            Ok::<_, Infallible>(
                                handle_admin_request(req, service_registry).await,
                            )
>>>>>>> origin/main
                        }
                    });

                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        eprintln!("Error serving admin connection: {:?}", err);
                    }
                });
            }
        });

        self.server_handle = Some(server_handle);
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

async fn handle_admin_request(
    req: Request<hyper::body::Incoming>,
    service_registry: Arc<RwLock<ServiceRegistry>>,
) -> Response<Full<Bytes>> {
    let admin_token = env::var("APICENTRIC_ADMIN_TOKEN").ok();

    if let Some(admin_token) = admin_token {
<<<<<<< HEAD
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());
=======
        let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
>>>>>>> origin/main

        if auth_header.is_none() || !auth_header.unwrap().starts_with("Bearer ") {
            let mut unauthorized = Response::new(Full::new(Bytes::from("Unauthorized")));
            *unauthorized.status_mut() = StatusCode::UNAUTHORIZED;
            return unauthorized;
        }

        let token = auth_header.unwrap().trim_start_matches("Bearer ");

        if token != admin_token {
            let mut forbidden = Response::new(Full::new(Bytes::from("Forbidden")));
            *forbidden.status_mut() = StatusCode::FORBIDDEN;
            return forbidden;
        }
    }

    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/apicentric-admin/logs") => {
            let services = {
                let registry = service_registry.read().await;
                registry.list_services().await
            };
            let mut all_logs = vec![];
            for service_info in services {
                let service = {
                    let registry = service_registry.read().await;
                    registry.get_service(&service_info.name).cloned()
                };
                if let Some(service) = service {
                    let logs = service.read().await.get_logs(100).await;
                    all_logs.extend(logs);
                }
            }
            match serde_json::to_string(&all_logs) {
                Ok(body) => Response::new(Full::new(Bytes::from(body))),
                Err(_) => {
                    let mut error = Response::new(Full::new(Bytes::from("Internal Server Error")));
                    *error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    error
                }
            }
        }
        _ => {
            let mut not_found = Response::new(Full::new(Bytes::from("Not Found")));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            not_found
        }
    }
}
