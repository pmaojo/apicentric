#[cfg(feature = "graphql")]
use async_graphql::Request as GraphQLRequest;
#[cfg(feature = "graphql")]
use async_graphql_parser::parse_schema;

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::config::GraphQLConfig;
use crate::simulator::template::TemplateEngine;
use bytes::Bytes;
use http_body_util::Full;
use hyper::Response;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::state::ServiceState;

/// Holds loaded GraphQL schema and mock templates
#[derive(Debug, Clone)]
pub struct GraphQLMocks {
    pub schema: String,
    pub mocks: HashMap<String, String>,
}

/// Load GraphQL schema and mock templates from configuration
#[cfg(feature = "graphql")]
pub fn load_graphql_mocks(gql_cfg: &GraphQLConfig) -> ApicentricResult<GraphQLMocks> {
    let schema = fs::read_to_string(&gql_cfg.schema_path).map_err(|e| {
        ApicentricError::config_error(
            format!("Failed to read GraphQL schema {}: {}", gql_cfg.schema_path, e),
            Some("Check that the schema file exists and is readable"),
        )
    })?;

    if let Err(e) = parse_schema(&schema) {
        return Err(ApicentricError::config_error(
            format!("Invalid GraphQL schema: {}", e),
            Some("Ensure the schema is valid SDL"),
        ));
    }

    let mut mocks = HashMap::new();
    for (op, path) in &gql_cfg.mocks {
        let tmpl = fs::read_to_string(path).map_err(|e| {
            ApicentricError::config_error(
                format!("Failed to read GraphQL mock template {}: {}", path, e),
                Some("Check template file path"),
            )
        })?;
        mocks.insert(op.clone(), tmpl);
    }

    Ok(GraphQLMocks { schema, mocks })
}

/// Load GraphQL schema and mock templates from configuration (GraphQL feature disabled)
#[cfg(not(feature = "graphql"))]
pub fn load_graphql_mocks(_gql_cfg: &GraphQLConfig) -> ApicentricResult<GraphQLMocks> {
    Err(ApicentricError::config_error(
        "GraphQL support is not enabled",
        Some("Rebuild with --features graphql to enable GraphQL support"),
    ))
}

/// Handle a GraphQL request if applicable
#[cfg(feature = "graphql")]
pub async fn handle_graphql_request(
    gql: &GraphQLMocks,
    method: &str,
    relative_path: &str,
    body_bytes: &[u8],
    query_params: &HashMap<String, String>,
    headers: &HashMap<String, String>,
    template_engine: &TemplateEngine,
    state: &Arc<RwLock<ServiceState>>,
    _service_name: &str,
    _path: &str,
) -> Option<(Response<Full<Bytes>>, u16)> {
    if relative_path != "/graphql" {
        return None;
    }

    if method == "GET" {
        let resp = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain")
            .body(Full::new(Bytes::from(gql.schema.clone())))
            .unwrap();
        return Some((resp, 200));
    } else if method == "POST" {
        match serde_json::from_slice::<GraphQLRequest>(body_bytes) {
            Ok(req_data) => {
                if let Some(op) = req_data.operation_name.clone() {
                    if let Some(tmpl) = gql.mocks.get(&op) {
                        let state_guard = state.read().await;
                        let params = PathParameters::new();
                        let request_body: Option<Value> = serde_json::from_slice(body_bytes).ok();
                        let request_context = RequestContext::from_request_data(
                            method.to_string(),
                            relative_path.to_string(),
                            query_params.clone(),
                            headers.clone(),
                            request_body.clone(),
                        );
                        let template_context =
                            TemplateContext::new(&state_guard, &params, request_context);
                        match template_engine.render(tmpl, &template_context) {
                            Ok(body) => {
                                let resp = Response::builder()
                                    .status(StatusCode::OK)
                                    .header("content-type", "application/json")
                                    .body(Full::new(Bytes::from(body)))
                                    .unwrap();
                                return Some((resp, 200));
                            }
                            Err(e) => {
                                let resp = Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .header("content-type", "application/json")
                                    .body(Full::new(Bytes::from(format!(
                                        "{{\"error\":\"{}\"}}",
                                        e
                                    ))))
                                    .unwrap();
                                return Some((
                                    resp,
                                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                                ));
                            }
                        }
                    } else {
                        let resp = Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header("content-type", "application/json")
                            .body(Full::new(Bytes::from(format!(
                                "{{\"error\":\"Unknown operation {}\"}}",
                                op
                            ))))
                            .unwrap();
                        return Some((resp, StatusCode::BAD_REQUEST.as_u16()));
                    }
                } else {
                    let resp = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(
                            r#"{"error":"Missing operationName"}"#,
                        )))
                        .unwrap();
                    return Some((resp, StatusCode::BAD_REQUEST.as_u16()));
                }
            }
            Err(_) => {
                let resp = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"error":"Invalid GraphQL request"}"#,
                    )))
                    .unwrap();
                return Some((resp, StatusCode::BAD_REQUEST.as_u16()));
            }
        }
    }

    None
}

/// Handle a GraphQL request if applicable (GraphQL feature disabled)
#[cfg(not(feature = "graphql"))]
pub async fn handle_graphql_request(
    _gql: &GraphQLMocks,
    _method: &str,
    _relative_path: &str,
    _body_bytes: &[u8],
    _query_params: &HashMap<String, String>,
    _headers: &HashMap<String, String>,
    _template_engine: &TemplateEngine,
    _state: &Arc<RwLock<ServiceState>>,
    _service_name: &str,
    _path: &str,
) -> Option<(Response<Full<Bytes>>, u16)> {
    None
}
