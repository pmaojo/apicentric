//! Recording session management for the cloud API.
//!
//! This module provides a recording session manager that can start and stop
//! recording proxies, track captured requests, and generate service definitions.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ParameterDefinition, ParameterLocation, ResponseDefinition,
};
use crate::utils::validate_ssrf_url;
use hyper::{HeaderMap, Method};
use reqwest::Url;

/// A recording session that captures HTTP traffic.
pub struct RecordingSession {
    /// Unique session ID.
    pub session_id: String,
    /// Target URL being proxied.
    pub target_url: String,
    /// Port the proxy is listening on.
    pub proxy_port: u16,
    /// Captured endpoints.
    pub endpoints: Arc<Mutex<HashMap<(String, String), EndpointDefinition>>>,
    /// Handle to the proxy task.
    pub task_handle: JoinHandle<()>,
    /// Shutdown signal sender.
    pub shutdown_tx: tokio::sync::mpsc::Sender<()>,
}

/// Manager for recording sessions.
pub struct RecordingSessionManager {
    /// Currently active recording session.
    active_session: Arc<RwLock<Option<RecordingSession>>>,
}

impl Default for RecordingSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordingSessionManager {
    /// Creates a new recording session manager.
    pub fn new() -> Self {
        Self {
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    /// Starts a new recording session.
    pub async fn start_recording(
        &self,
        target_url: String,
        port: u16,
    ) -> ApicentricResult<(String, String, u16)> {
        // Check if there's already an active session
        let mut session_lock = self.active_session.write().await;
        if session_lock.is_some() {
            return Err(ApicentricError::runtime_error(
                "A recording session is already active",
                Some("Stop the current session before starting a new one"),
            ));
        }

        // Validate SSRF
        let (url, addr) = validate_ssrf_url(&target_url).await.map_err(|e| {
            ApicentricError::validation_error(e, Some("target_url"), None::<String>)
        })?;

        let session_id = Uuid::new_v4().to_string();
        let proxy_url = format!("http://0.0.0.0:{}", port);

        let endpoints: Arc<Mutex<HashMap<(String, String), EndpointDefinition>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

        // Clone for the task
        let endpoints_clone = endpoints.clone();
        let target_url_clone = url; // Use the parsed URL

        // Start the proxy task
        let task_handle = tokio::spawn(async move {
            if let Err(e) = run_recording_proxy(
                target_url_clone,
                addr,
                port,
                endpoints_clone,
                &mut shutdown_rx,
            )
            .await
            {
                eprintln!("Recording proxy error: {}", e);
            }
        });

        let session = RecordingSession {
            session_id: session_id.clone(),
            target_url: target_url.clone(),
            proxy_port: port,
            endpoints,
            task_handle,
            shutdown_tx,
        };

        *session_lock = Some(session);

        Ok((session_id, proxy_url, port))
    }

    /// Stops the active recording session and returns captured endpoints.
    pub async fn stop_recording(&self) -> ApicentricResult<(String, Vec<EndpointDefinition>)> {
        let mut session_lock = self.active_session.write().await;

        if let Some(session) = session_lock.take() {
            // Send shutdown signal
            let _ = session.shutdown_tx.send(()).await;

            // Wait a bit for graceful shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Abort the task if it's still running
            session.task_handle.abort();

            // Get the captured endpoints
            let endpoints_map = session.endpoints.lock().await;
            let endpoints: Vec<EndpointDefinition> = endpoints_map.values().cloned().collect();

            Ok((session.session_id, endpoints))
        } else {
            Err(ApicentricError::runtime_error(
                "No active recording session",
                Some("Start a recording session first"),
            ))
        }
    }

    /// Gets the status of the current recording session.
    pub async fn get_status(
        &self,
    ) -> (
        bool,
        Option<String>,
        Option<String>,
        Option<u16>,
        Option<String>,
        usize,
    ) {
        let session_lock = self.active_session.read().await;

        if let Some(session) = session_lock.as_ref() {
            let endpoints = session.endpoints.lock().await;
            let count = endpoints.len();

            (
                true,
                Some(session.session_id.clone()),
                Some(format!("http://0.0.0.0:{}", session.proxy_port)),
                Some(session.proxy_port),
                Some(session.target_url.clone()),
                count,
            )
        } else {
            (false, None, None, None, None, 0)
        }
    }
}

/// Runs the recording proxy server.
async fn run_recording_proxy(
    target_url: Url,
    target_addr: SocketAddr,
    port: u16,
    endpoints: Arc<Mutex<HashMap<(String, String), EndpointDefinition>>>,
    shutdown_rx: &mut tokio::sync::mpsc::Receiver<()>,
) -> ApicentricResult<()> {
    use bytes::Bytes;
    use http_body_util::{BodyExt, Full};
    use hyper::body::Incoming;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use tokio::net::TcpListener;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let host = target_url
        .host_str()
        .ok_or_else(|| ApicentricError::runtime_error("Target URL has no host", None::<String>))?;

    // Create reqwest client with pinned IP to prevent DNS rebinding
    let client = reqwest::Client::builder()
        .resolve(host, target_addr)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to build proxy client: {}", e),
                None::<String>,
            )
        })?;

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to bind recording proxy: {}", e),
            None::<String>,
        )
    })?;

    println!(
        "🔴 Recording proxy listening on http://{} forwarding to {}",
        addr, target_url
    );

    loop {
        tokio::select! {
            res = listener.accept() => {
                let (stream, _) = res.map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Accept error: {}", e),
                        None::<String>,
                    )
                })?;

                let io = TokioIo::new(stream);
                let client = client.clone();
                let target_url = target_url.clone();
                let endpoints = endpoints.clone();

                tokio::spawn(async move {
                    let service = service_fn(move |req: Request<Incoming>| {
                        let client = client.clone();
                        let target_url = target_url.clone();
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

                            // Construct target URL using string formatting to preserve path behavior
                            let base = target_url.to_string().trim_end_matches('/').to_string();
                            let url_str = format!("{}{}", base, path_and_query);
                            let url = match Url::parse(&url_str) {
                                Ok(u) => u,
                                Err(e) => {
                                    let mut err_resp: Response<Full<Bytes>> =
                                        Response::new(Full::from(Bytes::from(e.to_string())));
                                    *err_resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
                                    return Ok::<_, Infallible>(err_resp);
                                }
                            };

                            // Build request with reqwest
                            // Note: reqwest 0.11 uses http 0.2, but we are using http 1.0 (via hyper 1.0)
                            // So we need to convert types manually.
                            let reqwest_method = match reqwest::Method::from_bytes(method.as_str().as_bytes()) {
                                Ok(m) => m,
                                Err(_) => {
                                    let mut err_resp: Response<Full<Bytes>> =
                                        Response::new(Full::from(Bytes::from("Invalid method")));
                                    *err_resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
                                    return Ok::<_, Infallible>(err_resp);
                                }
                            };

                            let mut request_builder = client.request(reqwest_method, url);

                            // Copy headers, excluding Host to allow reqwest to set it correctly
                            for (key, value) in headers.iter() {
                                if key.as_str().eq_ignore_ascii_case("host") {
                                    continue;
                                }

                                // Convert http 1.0 headers to reqwest (http 0.2) headers
                                let key_bytes = key.as_str().as_bytes();
                                let reqwest_key = match reqwest::header::HeaderName::from_bytes(key_bytes) {
                                    Ok(k) => k,
                                    Err(_) => continue,
                                };

                                let value_bytes = value.as_bytes();
                                let reqwest_value = match reqwest::header::HeaderValue::from_bytes(value_bytes) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };

                                request_builder = request_builder.header(reqwest_key, reqwest_value);
                            }

                            request_builder = request_builder.body(req_body.clone());

                            let resp = match request_builder.send().await {
                                Ok(r) => r,
                                Err(e) => {
                                    let mut err_resp = Response::new(Full::from(Bytes::from(e.to_string())));
                                    *err_resp.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                                    return Ok::<_, Infallible>(err_resp);
                                }
                            };

                            // Convert reqwest response back to hyper 1.0 types
                            let status = hyper::StatusCode::from_u16(resp.status().as_u16()).unwrap_or(hyper::StatusCode::INTERNAL_SERVER_ERROR);

                            let mut hyper_headers = hyper::HeaderMap::new();
                            for (k, v) in resp.headers() {
                                let key = match hyper::header::HeaderName::from_bytes(k.as_str().as_bytes()) {
                                    Ok(k) => k,
                                    Err(_) => continue,
                                };
                                let value = match hyper::header::HeaderValue::from_bytes(v.as_bytes()) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };
                                hyper_headers.insert(key, value);
                            }

                            let resp_bytes = match resp.bytes().await {
                                Ok(b) => b,
                                Err(e) => {
                                    let mut err_resp: Response<Full<Bytes>> =
                                        Response::new(Full::from(Bytes::from(e.to_string())));
                                    *err_resp.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                                    return Ok::<_, Infallible>(err_resp);
                                }
                            };

                            let content_type = hyper_headers
                                .get(hyper::header::CONTENT_TYPE)
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("application/json")
                                .to_string();

                            {
                                let mut map = endpoints.lock().await;
                                upsert_recorded_endpoint(
                                    &mut map,
                                    &method,
                                    &path,
                                    status.as_u16(),
                                    &content_type,
                                    String::from_utf8_lossy(&resp_bytes).into(),
                                    &hyper_headers,
                                );
                            }

                            let mut client_resp: Response<Full<Bytes>> =
                                Response::new(Full::from(resp_bytes));
                            *client_resp.status_mut() = status;
                            *client_resp.headers_mut() = hyper_headers;
                            Ok::<_, Infallible>(client_resp)
                        }
                    });

                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        eprintln!("Proxy connection error: {err}");
                    }
                });
            },
            _ = shutdown_rx.recv() => {
                println!("🛑 Recording proxy shutting down");
                break;
            }
        }
    }

    Ok(())
}

const ORIGINAL_PATH_PARAMS_HEADER: &str = "x-apicentric-recorded-path-params";

fn upsert_recorded_endpoint(
    map: &mut HashMap<(String, String), EndpointDefinition>,
    method: &Method,
    path: &str,
    status: u16,
    content_type: &str,
    body: String,
    headers: &HeaderMap,
) {
    let (normalized_path, parameter_defs, recorded_values) = parameterize_path(path);
    let key = (method.to_string(), normalized_path.clone());
    let entry = map.entry(key).or_insert_with(|| EndpointDefinition {
        kind: EndpointKind::Http,
        method: method.to_string(),
        path: normalized_path.clone(),
        header_match: None,
        description: None,
        parameters: if parameter_defs.is_empty() {
            None
        } else {
            Some(parameter_defs.clone())
        },
        request_body: None,
        responses: HashMap::new(),
        scenarios: None,
        stream: None,
    });

    entry.path = normalized_path;
    if !parameter_defs.is_empty() {
        entry.parameters = Some(parameter_defs.clone());
    }

    let mut response_headers = header_map_to_hash_map(headers);
    if let Some(metadata) = merge_recorded_values(
        entry
            .responses
            .get(&status)
            .and_then(|resp| resp.headers.as_ref())
            .and_then(|map| map.get(ORIGINAL_PATH_PARAMS_HEADER)),
        &recorded_values,
    ) {
        response_headers.insert(ORIGINAL_PATH_PARAMS_HEADER.to_string(), metadata);
    }

    let headers_option = if response_headers.is_empty() {
        None
    } else {
        Some(response_headers)
    };

    entry.responses.insert(
        status,
        ResponseDefinition {
            condition: None,
            content_type: content_type.to_string(),
            body,
            schema: None,
            script: None,
            headers: headers_option,
            side_effects: None,
        },
    );
}

fn parameterize_path(path: &str) -> (String, Vec<ParameterDefinition>, Vec<(String, String)>) {
    if path == "/" {
        return ("/".to_string(), Vec::new(), Vec::new());
    }

    let mut normalized_segments = Vec::new();
    let mut parameters = Vec::new();
    let mut recorded_values = Vec::new();
    let mut param_index = 1;

    for segment in path.split('/').filter(|s| !s.is_empty()) {
        if is_variable_segment(segment) {
            let param_name = format!("param{}", param_index);
            param_index += 1;
            normalized_segments.push(format!("{{{}}}", param_name));
            parameters.push(ParameterDefinition {
                name: param_name.clone(),
                location: ParameterLocation::Path,
                param_type: "string".to_string(),
                required: true,
                description: None,
            });
            recorded_values.push((param_name, segment.to_string()));
        } else {
            normalized_segments.push(segment.to_string());
        }
    }

    let normalized_path = if normalized_segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", normalized_segments.join("/"))
    };

    (normalized_path, parameters, recorded_values)
}

fn header_map_to_hash_map(headers: &HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            map.insert(name.to_string(), value_str.to_string());
        }
    }
    map
}

fn merge_recorded_values(
    existing: Option<&String>,
    new_values: &[(String, String)],
) -> Option<String> {
    if new_values.is_empty() {
        return existing.cloned();
    }

    let mut aggregated: HashMap<String, Vec<String>> = existing
        .and_then(|raw| serde_json::from_str::<HashMap<String, Vec<String>>>(raw).ok())
        .or_else(|| {
            existing
                .and_then(|raw| serde_json::from_str::<HashMap<String, String>>(raw).ok())
                .map(|map| {
                    map.into_iter()
                        .map(|(k, v)| (k, vec![v]))
                        .collect::<HashMap<_, _>>()
                })
        })
        .unwrap_or_default();

    for (name, value) in new_values {
        let entry = aggregated.entry(name.clone()).or_default();
        if !entry.contains(value) {
            entry.push(value.clone());
        }
    }

    if aggregated.is_empty() {
        None
    } else {
        serde_json::to_string(&aggregated).ok()
    }
}

fn is_variable_segment(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }

    if segment.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    if uuid::Uuid::parse_str(segment).is_ok() {
        return true;
    }

    if segment.len() >= 16 && segment.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }

    let has_digit = segment.chars().any(|c| c.is_ascii_digit());
    let has_alpha = segment.chars().any(|c| c.is_ascii_alphabetic());
    let valid_chars = segment
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '~'));

    has_digit && has_alpha && valid_chars && segment.len() >= 4
}
