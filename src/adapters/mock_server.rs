use crate::{PulseError, PulseResult};
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{body::Incoming, Method, Request, Response, StatusCode};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, Clone)]
pub struct MockApiSpec {
    pub name: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub base_path: Option<String>,
    #[serde(default)]
    pub endpoints: Vec<MockEndpoint>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MockEndpoint {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub delay_ms: Option<u64>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub response: serde_yaml::Value,
}

impl Default for MockEndpoint {
    fn default() -> Self {
        Self {
            method: "GET".into(),
            path: "/".into(),
            status: 200,
            delay_ms: None,
            headers: HashMap::new(),
            response: serde_yaml::Value::Null,
        }
    }
}

pub struct MockServerState {
    endpoints: Vec<CompiledEndpoint>,
}

struct CompiledEndpoint {
    method: Method,
    matcher: EndpointMatcher,
    endpoint: MockEndpoint,
}

enum EndpointMatcher {
    Exact(String),
    Regex(Regex),
}

impl MockServerState {
    fn match_request(&self, method: &Method, path: &str) -> Option<&CompiledEndpoint> {
        self.endpoints.iter().find(|ce| {
            ce.method == *method
                && match &ce.matcher {
                    EndpointMatcher::Exact(p) => p == path,
                    EndpointMatcher::Regex(r) => r.is_match(path),
                }
        })
    }
}

pub struct MockServerBuilder {
    spec: MockApiSpec,
}
impl MockServerBuilder {
    pub fn new(spec: MockApiSpec) -> Self {
        Self { spec }
    }
}

impl MockServerBuilder {
    pub fn build(self) -> PulseResult<MockServerState> {
        let mut compiled = Vec::new();
        for ep in self.spec.endpoints {
            let method: Method = ep.method.parse().map_err(|_| {
                PulseError::config_error(
                    format!("Método HTTP inválido: {}", ep.method),
                    Some("Usa GET, POST, PUT, DELETE, PATCH"),
                )
            })?;
            let matcher = if ep.path.contains('{') || ep.path.contains('(') {
                // treat as regex style (OpenAPI like /users/{id}) -> convert {var} to [^/]+
                let re_str = ep.path.replace('{', "(?P<").replace('}', ">[^/]+)");
                let re = Regex::new(&format!("^{}$", re_str)).map_err(|e| {
                    PulseError::config_error(
                        format!("Regex inválido en path {}: {}", ep.path, e),
                        None::<String>,
                    )
                })?;
                EndpointMatcher::Regex(re)
            } else if ep.path.starts_with('^') {
                let re = Regex::new(&ep.path).map_err(|e| {
                    PulseError::config_error(
                        format!("Regex inválido en path {}: {}", ep.path, e),
                        None::<String>,
                    )
                })?;
                EndpointMatcher::Regex(re)
            } else {
                EndpointMatcher::Exact(ep.path.clone())
            };
            compiled.push(CompiledEndpoint {
                method,
                matcher,
                endpoint: ep,
            });
        }
        Ok(MockServerState {
            endpoints: compiled,
        })
    }
}

pub async fn load_spec(path: &Path) -> PulseResult<MockApiSpec> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PulseError::fs_error(
            format!("No se puede leer YAML {}: {}", path.display(), e),
            Some("Verifica permisos"),
        )
    })?;
    let spec: MockApiSpec = serde_yaml::from_str(&content).map_err(|e| {
        PulseError::config_error(
            format!("YAML inválido {}: {}", path.display(), e),
            Some("Valida formato YAML"),
        )
    })?;
    Ok(spec)
}

pub async fn run_mock_server(spec: MockApiSpec) -> PulseResult<()> {
    let port = spec.port.unwrap_or(7070);
    let state = Arc::new(RwLock::new(MockServerBuilder::new(spec).build()?));
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    println!("🦐 Pulse Mock API escuchando en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        PulseError::server_error(
            format!("No se puede ligar puerto {}: {}", addr, e),
            None::<String>,
        )
    })?;
    loop {
        let (stream, remote) = listener.accept().await.map_err(|e| {
            PulseError::server_error(format!("Fallo accept: {}", e), None::<String>)
        })?;
        let state_clone = state.clone();
        tokio::spawn(async move {
            let io = hyper_util::rt::tokio::TokioIo::new(stream);
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    io,
                    hyper::service::service_fn(move |req| {
                        let st = state_clone.clone();
                        async move { handle_request(st, req).await }
                    }),
                )
                .with_upgrades();
            if let Err(err) = conn.await {
                eprintln!("⚠️ conexión {} error: {}", remote, err);
            }
        });
    }
}

async fn handle_request(
    state: Arc<RwLock<MockServerState>>,
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let state_guard = state.read().await;
    if let Some(ce) = state_guard.match_request(&method, &path) {
        if let Some(d) = ce.endpoint.delay_ms {
            tokio::time::sleep(std::time::Duration::from_millis(d)).await;
        }
        let mut builder = Response::builder()
            .status(StatusCode::from_u16(ce.endpoint.status).unwrap_or(StatusCode::OK));
        for (k, v) in &ce.endpoint.headers {
            builder = builder.header(k, v);
        }
        let body_str = if ce.endpoint.response.is_null() {
            "".to_string()
        } else if ce.endpoint.response.is_mapping() || ce.endpoint.response.is_sequence() {
            serde_json::to_string(&ce.endpoint.response).unwrap_or_else(|_| "{}".into())
        } else {
            ce.endpoint.response.as_str().unwrap_or("").to_string()
        };
        Ok(builder
            .body(
                Full::new(Bytes::from(body_str))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(
                Full::new(Bytes::from(format!("No match {} {}", method, path)))
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap())
    }
}
