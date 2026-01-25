use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::{
    pki_types::{CertificateDer, ServerName, UnixTime},
    DigitallySignedStruct,
};

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{HeaderMap, Request, Response, Uri};
use hyper_rustls::HttpsConnectorBuilder;
use std::error::Error;
// use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ParameterDefinition, ParameterLocation, ResponseDefinition,
    ServerConfig, ServiceDefinition,
};

/// Trait for recording traffic through a proxy.
#[async_trait(?Send)]
pub trait RecordingProxy {
    async fn record(&self, target: &str, output_dir: PathBuf, port: u16) -> ApicentricResult<()>;
}

/// Default implementation of [`RecordingProxy`].
pub struct ProxyRecorder;

#[async_trait(?Send)]
impl RecordingProxy for ProxyRecorder {
    async fn record(&self, target: &str, output_dir: PathBuf, port: u16) -> ApicentricResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        // Create a custom TLS configuration that ignores certificate validation errors
        let tls = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(std::sync::Arc::new(NoCertificateVerification))
            .with_no_client_auth();

        let https = HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .build();

        let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(https);
        let endpoints: Arc<Mutex<HashMap<(String, String), EndpointDefinition>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            ApicentricError::runtime_error(
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
                    let (stream, _) = res.map_err(|e| ApicentricError::runtime_error(format!("Accept error: {}", e), None::<String>))?;
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

                                let target_trimmed = target.trim_end_matches('/');
                                let uri_string = format!("{}{}", target_trimmed, path_and_query);
                                let uri: Uri = uri_string.parse().unwrap();

                                let mut fwd_req = Request::new(Full::from(req_body.clone()));
                                *fwd_req.method_mut() = method.clone();
                                *fwd_req.uri_mut() = uri;
                                *fwd_req.headers_mut() = headers.clone();

                                let resp = match client.request(fwd_req).await {
                                    Ok(r) => r,
                                    Err(e) => {
                                        eprintln!("Proxy request failed to {}: {} (source: {:?})", uri_string, e, e.source());
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
                                    upsert_recorded_endpoint(
                                        &mut map,
                                        &method,
                                        &path,
                                        parts.status.as_u16(),
                                        &content_type,
                                        String::from_utf8_lossy(&resp_bytes).into(),
                                        &parts.headers,
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
            server: Some(ServerConfig {
                port: None,
                base_path: "/".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Some(map.values().cloned().collect()),
            graphql: None,
            behavior: None,
            #[cfg(feature = "iot")]
            twin: None,
            #[cfg(feature = "bluetooth")]
            bluetooth: None,
        };

        std::fs::create_dir_all(&output_dir).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to create output directory: {}", e),
                None::<String>,
            )
        })?;
        let yaml = serde_yaml::to_string(&service).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to serialize service definition: {}", e),
                None::<String>,
            )
        })?;
        let path = output_dir.join("recorded_service.yaml");
        std::fs::write(&path, yaml).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to write service file: {}", e),
                None::<String>,
            )
        })?;
        println!("\u{2705} Recorded interactions saved to {}", path.display());
        Ok(())
    }
}

const ORIGINAL_PATH_PARAMS_HEADER: &str = "x-apicentric-recorded-path-params";

fn upsert_recorded_endpoint(
    map: &mut HashMap<(String, String), EndpointDefinition>,
    method: &hyper::Method,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::config::{EndpointDefinition, ServerConfig, ServiceDefinition};
    use crate::simulator::service::ServiceInstance;
    use hyper::Method;
    use std::collections::HashMap as StdHashMap;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[test]
    fn parameterize_path_detects_dynamic_segments() {
        let (templated, params, values) = parameterize_path("/users/123");

        assert_eq!(templated, "/users/{param1}");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "param1");
        assert_eq!(values, vec![("param1".to_string(), "123".to_string())]);

        let (templated_other, _, other_values) = parameterize_path("/users/456");
        assert_eq!(templated_other, templated);
        assert_eq!(
            other_values,
            vec![("param1".to_string(), "456".to_string())]
        );
    }

    #[test]
    fn upsert_collapses_similar_paths_into_one_endpoint() {
        let mut map: HashMap<(String, String), EndpointDefinition> = HashMap::new();
        let method = Method::GET;
        let headers = HeaderMap::new();

        upsert_recorded_endpoint(
            &mut map,
            &method,
            "/users/123",
            200,
            "application/json",
            "{\"id\":123}".to_string(),
            &headers,
        );

        upsert_recorded_endpoint(
            &mut map,
            &method,
            "/users/456",
            200,
            "application/json",
            "{\"id\":456}".to_string(),
            &headers,
        );

        assert_eq!(map.len(), 1);
        let endpoint = map.values().next().unwrap();
        assert_eq!(endpoint.path, "/users/{param1}");
        let params = endpoint
            .parameters
            .as_ref()
            .expect("parameters should be recorded");
        assert_eq!(params[0].name, "param1");

        let response = endpoint
            .responses
            .get(&200)
            .expect("response for status code should exist");
        let headers_map = response
            .headers
            .as_ref()
            .expect("metadata headers should exist");
        let metadata = headers_map
            .get(ORIGINAL_PATH_PARAMS_HEADER)
            .expect("metadata header should be present");
        let parsed: HashMap<String, Vec<String>> = serde_json::from_str(metadata).unwrap();
        assert_eq!(
            parsed.get("param1"),
            Some(&vec!["123".to_string(), "456".to_string()])
        );
    }

    #[tokio::test]
    async fn replay_uses_parameterized_template() {
        let mut map: HashMap<(String, String), EndpointDefinition> = HashMap::new();
        let method = Method::GET;
        let headers = HeaderMap::new();

        upsert_recorded_endpoint(
            &mut map,
            &method,
            "/users/123",
            200,
            "application/json",
            "{\"id\":123}".to_string(),
            &headers,
        );

        upsert_recorded_endpoint(
            &mut map,
            &method,
            "/users/456",
            200,
            "application/json",
            "{\"id\":456}".to_string(),
            &headers,
        );

        let service = ServiceDefinition {
            name: "test".to_string(),
            version: None,
            description: None,
            server: Some(ServerConfig {
                port: None,
                base_path: "/".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Some(map.values().cloned().collect()),
            graphql: None,
            behavior: None,
            #[cfg(feature = "iot")]
            twin: None,
            #[cfg(feature = "bluetooth")]
            bluetooth: None,
        };

        let storage = Arc::new(crate::storage::sqlite::SqliteStorage::init_db(":memory:").unwrap());
        let (tx, _) = broadcast::channel(1);
        let instance = ServiceInstance::new(service, 9000, storage, tx).expect("service instance");

        let headers = StdHashMap::new();
        let route = instance
            .find_endpoint_with_params("GET", "/users/456", &headers)
            .expect("route should be found");

        assert_eq!(route.endpoint.path, "/users/{param1}");
        assert_eq!(route.path_params.get("param1"), Some(&"456".to_string()));
    }
}
