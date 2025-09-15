use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::errors::{PulseError, PulseResult};
use crate::simulator::config::{EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition};

/// Trait for recording traffic through a proxy.
#[async_trait(?Send)]
pub trait RecordingProxy {
    async fn record(&self, target: &str, output_dir: PathBuf, port: u16) -> PulseResult<()>;
}

/// Default implementation of [`RecordingProxy`].
pub struct ProxyRecorder;

#[async_trait(?Send)]
impl RecordingProxy for ProxyRecorder {
    async fn record(&self, target: &str, output_dir: PathBuf, port: u16) -> PulseResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let connector = HttpConnector::new();
        let client: Client<HttpConnector, Full<Bytes>> =
            Client::builder(TokioExecutor::new()).build(connector);
        let endpoints: Arc<Mutex<HashMap<(String, String), EndpointDefinition>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to bind recording proxy: {}", e),
                None::<String>,
            )
        })?;

        println!(
            "\u{1F534} Recording proxy listening on http://{} forwarding to {}",
            addr, target
        );

        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (stream, _) = res.map_err(|e| PulseError::runtime_error(format!("Accept error: {}", e), None::<String>))?;
                    let io = TokioIo::new(stream);
                    let client = client.clone();
                    let target = target.to_string();
                    let endpoints = endpoints.clone();
                    tokio::spawn(async move {
                        let service = service_fn(move |req: Request<Incoming>| {
                            let client = client.clone();
                            let target = target.clone();
                            let endpoints = endpoints.clone();
                            async move {
                                let method = req.method().clone();
                                let headers = req.headers().clone();
                                let path = req.uri().path().to_string();
                                let path_and_query = req
                                    .uri()
                                    .path_and_query()
                                    .map(|pq| pq.as_str().to_string())
                                    .unwrap_or_else(|| path.clone());

                                let req_body = match BodyExt::collect(req.into_body()).await {
                                    Ok(col) => col.to_bytes(),
                                    Err(e) => {
                                        let mut err_resp: Response<Full<Bytes>> =
                                            Response::new(Full::from(Bytes::from(e.to_string())));
                                        *err_resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
                                        return Ok::<_, Infallible>(err_resp);
                                    }
                                };

                                let uri: Uri = format!("{}{}", target, path_and_query)
                                    .parse()
                                    .unwrap();

                                let mut fwd_req = Request::new(Full::from(req_body.clone()));
                                *fwd_req.method_mut() = method.clone();
                                *fwd_req.uri_mut() = uri;
                                *fwd_req.headers_mut() = headers.clone();

                                let resp = match client.request(fwd_req).await {
                                    Ok(r) => r,
                                    Err(e) => {
                                        let mut err_resp = Response::new(Full::from(Bytes::from(e.to_string())));
                                        *err_resp.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                                        return Ok::<_, Infallible>(err_resp);
                                    }
                                };
                                let (parts, body) = resp.into_parts();
                                let resp_bytes = match BodyExt::collect(body).await {
                                    Ok(col) => col.to_bytes(),
                                    Err(e) => {
                                        let mut err_resp: Response<Full<Bytes>> =
                                            Response::new(Full::from(Bytes::from(e.to_string())));
                                        *err_resp.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                                        return Ok::<_, Infallible>(err_resp);
                                    }
                                };

                                let content_type = parts
                                    .headers
                                    .get(hyper::header::CONTENT_TYPE)
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("application/json")
                                    .to_string();
                                {
                                    let mut map = endpoints.lock().await;
                                    let key = (method.to_string(), path.clone());
                                    let entry = map.entry(key).or_insert_with(|| EndpointDefinition {
                                        kind: EndpointKind::Http,
                                        method: method.to_string(),
                                        path: path.clone(),
                                        header_match: None,
                                        description: None,
                                        parameters: None,
                                        request_body: None,
                                        responses: HashMap::new(),
                                        scenarios: None,
                                        stream: None,
                                    });
                                    entry.responses.insert(
                                        parts.status.as_u16(),
                                        ResponseDefinition {
                                            condition: None,
                                            content_type: content_type.clone(),
                                            body: String::from_utf8_lossy(&resp_bytes).into(),
                                            script: None,
                                            headers: None,
                                            side_effects: None,
                                        },
                                    );
                                }

                                let mut client_resp: Response<Full<Bytes>> =
                                    Response::new(Full::from(resp_bytes));
                                *client_resp.status_mut() = parts.status;
                                *client_resp.headers_mut() = parts.headers;
                                Ok::<_, Infallible>(client_resp)
                            }
                        });
                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                            eprintln!("Proxy connection error: {err}");
                        }
                    });
                },
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        let map = endpoints.lock().await;
        let service = ServiceDefinition {
            name: "recorded_service".to_string(),
            version: None,
            description: Some("Recorded service".to_string()),
            server: ServerConfig {
                port: None,
                base_path: "/".to_string(),
                proxy_base_url: None,
                cors: None,
            },
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: map.values().cloned().collect(),
            graphql: None,
            behavior: None,
        };

        std::fs::create_dir_all(&output_dir).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to create output directory: {}", e),
                None::<String>,
            )
        })?;
        let yaml = serde_yaml::to_string(&service).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to serialize service definition: {}", e),
                None::<String>,
            )
        })?;
        let path = output_dir.join("recorded_service.yaml");
        std::fs::write(&path, yaml).map_err(|e| {
            PulseError::runtime_error(
                format!("Failed to write service file: {}", e),
                None::<String>,
            )
        })?;
        println!("\u{2705} Recorded interactions saved to {}", path.display());
        Ok(())
    }
}

