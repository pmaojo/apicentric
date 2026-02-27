use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::config::ResponseDefinition;
use crate::simulator::log::RequestLogEntry;
use crate::simulator::scripting::ScriptingEngine;
use crate::simulator::service::graphql::{handle_graphql_request, GraphQLMocks};
use crate::simulator::service::state::ServiceState;
use crate::simulator::service::{response_processor, scenario_matcher, ServiceInstance};
use crate::simulator::template::{RequestContext, TemplateContext, TemplateEngine};
use crate::storage::Storage;
use bytes::Bytes;
use http_body_util::Full;
use hyper::header::HOST;
use hyper::{Request, Response, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::RwLock;

pub struct HttpHandler;

impl HttpHandler {
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_request(
        req: Request<hyper::body::Incoming>,
        definition: Arc<StdRwLock<crate::simulator::config::ServiceDefinition>>,
        state: Arc<RwLock<ServiceState>>,
        template_engine: Arc<TemplateEngine>,
        scripting_engine: Arc<ScriptingEngine>,
        active_scenario: Arc<RwLock<Option<String>>>,
        graphql: Option<Arc<GraphQLMocks>>,
        storage: Arc<dyn Storage>,
    ) -> ApicentricResult<Response<Full<Bytes>>> {
        let (service_name, base_path, endpoints, cors_cfg, proxy_base_url, record_unknown) = {
            let def = definition.read().unwrap();
            let (base_path, cors_cfg, proxy_cfg, record_unknown) = if let Some(server) = &def.server
            {
                (
                    server.base_path.clone(),
                    server.cors.clone(),
                    server.proxy_base_url.clone(),
                    server.record_unknown,
                )
            } else {
                ("/".to_string(), None, None, false)
            };

            (
                def.name.clone(),
                base_path,
                def.endpoints.clone().unwrap_or_default(),
                cors_cfg,
                proxy_cfg,
                record_unknown,
            )
        };

        let (parts, body) = req.into_parts();
        let method = parts.method.as_str();
        let path = parts.uri.path();

        // Log incoming request
        Self::record_log(
            &state,
            &service_name,
            None,
            "DEBUG",
            &format!(
                "Request Origin: {}",
                parts
                    .headers
                    .get("origin")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("none")
            ),
            200,
            None,
        )
        .await;

        // Parse query parameters
        let query_params = parts
            .uri
            .query()
            .map(|q| {
                q.split('&')
                    .filter_map(|param| {
                        let mut parts = param.split('=');
                        match (parts.next(), parts.next()) {
                            (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                            _ => None,
                        }
                    })
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        // Parse headers
        let headers: HashMap<String, String> = parts
            .headers
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();

        // Handle CORS preflight
        if method == "OPTIONS" {
            let origin = headers.get("origin").cloned().unwrap_or_default();

            let allow_origin = match &cors_cfg {
                Some(cfg) => {
                    if cfg.origins.iter().any(|o| o == "*") {
                        "*".to_string()
                    } else if cfg.origins.iter().any(|o| o.eq_ignore_ascii_case(&origin)) {
                        origin.clone()
                    } else {
                        "*".to_string()
                    }
                }
                None => "*".to_string(),
            };
            let req_headers = headers
                .get("access-control-request-headers")
                .cloned()
                .unwrap_or_else(|| {
                    cors_cfg
                        .as_ref()
                        .and_then(|c| c.headers.clone())
                        .map(|v| v.join(", "))
                        .unwrap_or_else(|| "Content-Type, Authorization".to_string())
                });
            let allow_methods = cors_cfg
                .as_ref()
                .and_then(|c| c.methods.clone())
                .map(|v| v.join(", "))
                .unwrap_or_else(|| "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string());

            let resp = Response::builder()
                .status(StatusCode::NO_CONTENT)
                .header("access-control-allow-origin", &allow_origin)
                .header("access-control-allow-methods", &allow_methods)
                .header("access-control-allow-headers", &req_headers)
                .header("access-control-max-age", "86400")
                .body(Full::new(Bytes::from_static(b"")))
                .map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to build CORS preflight response: {}", e),
                        None::<String>,
                    )
                })?;

            Self::record_log(
                &state,
                &service_name,
                None,
                "DEBUG",
                "CORS preflight response sent",
                204,
                None,
            )
            .await;
            Self::record_log(
                &state,
                &service_name,
                None,
                method,
                path,
                StatusCode::NO_CONTENT.as_u16(),
                None,
            )
            .await;
            return Ok(resp);
        }

        // Parse request body if present
        let body_bytes = match http_body_util::BodyExt::collect(body).await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                let resp = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"error": "Failed to read request body"}"#,
                    )))
                    .map_err(|e| {
                        ApicentricError::runtime_error(
                            format!("Failed to build bad request response: {}", e),
                            None::<String>,
                        )
                    })?;
                Self::record_log(
                    &state,
                    &service_name,
                    None,
                    method,
                    path,
                    StatusCode::BAD_REQUEST.as_u16(),
                    None,
                )
                .await;
                return Ok(resp);
            }
        };

        let request_body = if !body_bytes.is_empty() {
            let body_str = String::from_utf8_lossy(&body_bytes);
            Self::record_log(
                &state,
                &service_name,
                None,
                "DEBUG",
                &format!("Request body: {}", body_str),
                200,
                None,
            )
            .await;

            // Determine content type
            let content_type = parts
                .headers
                .get(hyper::header::CONTENT_TYPE)
                .and_then(|hv| hv.to_str().ok())
                .unwrap_or("")
                .to_lowercase();

            if content_type.contains("application/x-www-form-urlencoded") {
                // Parse form-encoded body
                let mut map = serde_json::Map::new();
                for (k, v) in url::form_urlencoded::parse(body_str.as_bytes()) {
                    map.insert(k.to_string(), Value::String(v.into_owned()));
                }
                Some(Value::Object(map))
            } else {
                // Try to parse as JSON
                serde_json::from_str::<Value>(&body_str).ok()
            }
        } else {
            None
        };

        // Remove base path from request path if it matches
        let relative_path = if path.starts_with(&base_path) {
            &path[base_path.len()..]
        } else {
            path
        };

        // Ensure relative path starts with '/'
        let relative_path = if relative_path.is_empty() || !relative_path.starts_with('/') {
            format!("/{}", relative_path.trim_start_matches('/'))
        } else {
            relative_path.to_string()
        };

        // Handle GraphQL endpoint if configured
        if let Some(gql) = &graphql {
            if let Some((resp, status)) = handle_graphql_request(
                gql,
                method,
                &relative_path,
                &body_bytes,
                &query_params,
                &headers,
                &template_engine,
                &state,
                &service_name,
                path,
            )
            .await
            {
                Self::record_log(&state, &service_name, None, method, path, status, None).await;
                return Ok(resp);
            }
        }

        // Internal logs endpoint
        if method == "GET" && relative_path == "/__apicentric/logs" {
            let limit = query_params
                .get("limit")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(100);
            let method_filter = query_params.get("method").map(|s| s.as_str());
            let route_filter = query_params.get("route").map(|s| s.as_str());
            let status_filter = query_params
                .get("status")
                .and_then(|v| v.parse::<u16>().ok());
            let logs = {
                let state = state.read().await;
                state.query_logs(
                    Some(&service_name),
                    route_filter,
                    method_filter,
                    status_filter,
                    limit,
                )
            };
            let body = serde_json::to_string(&logs)?;
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(body)))
                .map_err(|e| {
                    ApicentricError::runtime_error(
                        format!("Failed to build logs response: {}", e),
                        None::<String>,
                    )
                })?;
            Self::record_log(&state, &service_name, None, method, path, 200, None).await;
            return Ok(resp);
        }

        // Find matching endpoint with parameter extraction
        let route_match = ServiceInstance::find_endpoint_with_params_static(
            &endpoints,
            method,
            &relative_path,
            &headers,
        );

        match route_match {
            Some(route_match) => {
                // Evaluate conditions to find the right response
                let mut selected_response: Option<ResponseDefinition> = None;
                let mut selected_status = 200u16;

                // Try to match explicit or rotating scenarios
                let active = active_scenario.read().await.clone();
                if let Some((status, resp)) = scenario_matcher::match_scenario(
                    &route_match.endpoint,
                    &state,
                    route_match.endpoint_index,
                    active,
                    &query_params,
                    &headers,
                    &request_body,
                )
                .await
                {
                    selected_status = status;
                    selected_response = Some(resp);
                } else {
                    // Try to find a response with a matching condition
                    for (status_code, response_def) in &route_match.endpoint.responses {
                        if let Some(ref condition) = response_def.condition {
                            // Create template context for condition evaluation
                            let state_guard = state.read().await;
                            let request_context = RequestContext::from_request_data(
                                method.to_string(),
                                relative_path.clone(),
                                query_params.clone(),
                                headers.clone(),
                                request_body.clone(),
                            );

                            let template_context = TemplateContext::new(
                                &state_guard,
                                &route_match.path_params,
                                request_context,
                            );

                            // Evaluate condition
                            match template_engine.render(condition, &template_context) {
                                Ok(result) => {
                                    // Check if condition evaluates to truthy
                                    let is_truthy = !result.trim().is_empty()
                                        && result.trim() != "null"
                                        && result.trim() != "false";

                                    if is_truthy {
                                        selected_response = Some(response_def.clone());
                                        selected_status = *status_code;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    log::warn!("Condition evaluation error: {}", e);
                                }
                            }
                        } else {
                            // No condition, use this response as fallback
                            if selected_response.is_none() {
                                selected_response = Some(response_def.clone());
                                selected_status = *status_code;
                            }
                        }
                    }

                    // If no conditional response matched, use default (200 if available)
                    if selected_response.is_none() {
                        if let Some(default_response) = route_match.endpoint.responses.get(&200) {
                            selected_response = Some(default_response.clone());
                            selected_status = 200;
                        } else if let Some((status, response)) =
                            route_match.endpoint.responses.iter().next()
                        {
                            selected_response = Some(response.clone());
                            selected_status = *status;
                        }
                    }
                }

                if let Some(response_def) = selected_response {
                    let request_context = RequestContext::from_request_data(
                        method.to_string(),
                        relative_path.clone(),
                        query_params.clone(),
                        headers.clone(),
                        request_body.clone(),
                    );

                    let mut script_body_override = None;
                    if let Some(ref script_path) = response_def.script {
                        match Self::execute_script(
                            script_path.as_path(),
                            &state,
                            &scripting_engine,
                            &route_match.path_params,
                            &request_context,
                        )
                        .await
                        {
                            Ok(result) => {
                                if !result.is_null() {
                                    script_body_override = Some(result);
                                }
                            }
                            Err(e) => {
                                log::warn!("Script execution error: {}", e);
                            }
                        }
                    }

                    let state_guard = state.read().await;
                    let template_context = TemplateContext::new(
                        &state_guard,
                        &route_match.path_params,
                        request_context,
                    );
                    drop(state_guard);

                    let response_body = if let Some(body_v) = script_body_override {
                        if body_v.is_string() {
                            body_v.as_str().unwrap().to_string()
                        } else {
                            serde_json::to_string(&body_v)
                                .unwrap_or_else(|_| response_def.body.clone())
                        }
                    } else {
                        response_def.body.clone()
                    };
                    let processed_body = if response_body.contains("{{") {
                        match response_processor::process_response_body_template(
                            &response_body,
                            &template_context,
                            &template_engine,
                            &service_name,
                            method,
                            path,
                        ) {
                            Ok(rendered) => rendered,
                            Err(e) => {
                                return Err(ApicentricError::runtime_error(
                                    format!(
                                        "Failed to process template for {} {} in service '{}': {}",
                                        method, path, service_name, e
                                    ),
                                    Some("Check template syntax and fixture availability"),
                                ));
                            }
                        }
                    } else {
                        response_body
                    };

                    if let Some(ref side_effects) = response_def.side_effects {
                        let mut state_guard = state.write().await;
                        for side_effect in side_effects {
                            if let Err(e) = response_processor::process_side_effect(
                                side_effect,
                                &mut state_guard,
                                &template_context,
                                &template_engine,
                            ) {
                                log::warn!("Side effect processing error: {}", e);
                            }
                        }
                    }

                    let mut response = Response::builder()
                        .status(StatusCode::from_u16(selected_status).unwrap_or(StatusCode::OK))
                        .header("content-type", &response_def.content_type);

                    if let Some(ref headers_map) = response_def.headers {
                        for (key, value) in headers_map {
                            let header_value = if value.contains("{{") {
                                match template_engine.render(value, &template_context) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        log::warn!("Header template rendering error: {}", e);
                                        value.clone()
                                    }
                                }
                            } else {
                                value.clone()
                            };
                            response = response.header(key, header_value);
                        }
                    }

                    // Add CORS headers if enabled
                    if let Some(cfg) = &cors_cfg {
                        let origin_hdr = headers.get("origin").cloned().unwrap_or_default();
                        let allow_origin = if cfg.origins.iter().any(|o| o == "*") {
                            "*".to_string()
                        } else if cfg
                            .origins
                            .iter()
                            .any(|o| o.eq_ignore_ascii_case(&origin_hdr))
                        {
                            origin_hdr.clone()
                        } else {
                            "*".to_string()
                        };
                        let allow_methods = cfg
                            .methods
                            .clone()
                            .map(|v| v.join(", "))
                            .unwrap_or_else(|| {
                                "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string()
                            });
                        let allow_headers = cfg
                            .headers
                            .clone()
                            .map(|v| v.join(", "))
                            .unwrap_or_else(|| "Content-Type, Authorization".to_string());

                        response = response
                            .header("access-control-allow-origin", &allow_origin)
                            .header("access-control-allow-methods", &allow_methods)
                            .header("access-control-allow-headers", &allow_headers);
                    } else {
                        response = response.header("access-control-allow-origin", "*");
                    }

                    let final_response = response
                        .body(Full::new(Bytes::from(processed_body)))
                        .map_err(|e| {
                            ApicentricError::runtime_error(
                                format!("Failed to build response body: {}", e),
                                None::<String>,
                            )
                        })?;

                    Self::record_log(
                        &state,
                        &service_name,
                        Some(route_match.endpoint_index),
                        method,
                        path,
                        selected_status,
                        None,
                    )
                    .await;
                    Ok(final_response)
                } else {
                    // No response definition found
                    let resp = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(
                            r#"{"error": "No response definition found"}"#,
                        )))
                        .map_err(|e| {
                            ApicentricError::runtime_error(
                                format!("Failed to build error response: {}", e),
                                None::<String>,
                            )
                        })?;
                    Self::record_log(
                        &state,
                        &service_name,
                        Some(route_match.endpoint_index),
                        method,
                        path,
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        None,
                    )
                    .await;
                    Ok(resp)
                }
            }
            None => {
                // No matching endpoint found
                if let Some(base_url) = proxy_base_url {
                    // Forward request to proxy target
                    let query = parts
                        .uri
                        .query()
                        .map(|q| format!("?{}", q))
                        .unwrap_or_default();
                    let target_url = format!(
                        "{}{}{}",
                        base_url.trim_end_matches('/'),
                        relative_path,
                        query
                    );

                    let client = reqwest::Client::new();
                    let req_method = reqwest::Method::from_bytes(method.as_bytes())
                        .unwrap_or(reqwest::Method::GET);
                    let mut builder = client.request(req_method, target_url);

                    // Copy headers except host
                    for (name, value) in parts.headers.iter() {
                        if name != HOST {
                            if let Ok(v) = value.to_str() {
                                builder = builder.header(name.as_str(), v);
                            }
                        }
                    }

                    if !body_bytes.is_empty() {
                        builder = builder.body(body_bytes.clone());
                    }

                    match builder.send().await {
                        Ok(resp) => {
                            let status = StatusCode::from_u16(resp.status().as_u16())
                                .unwrap_or(StatusCode::OK);
                            let headers = resp.headers().clone();
                            let bytes = resp.bytes().await.unwrap_or_else(|_| Bytes::new());
                            let mut response = Response::builder().status(status);
                            for (name, value) in headers.iter() {
                                if let Ok(v) = value.to_str() {
                                    response = response.header(name.as_str(), v);
                                }
                            }
                            let final_resp = response.body(Full::new(bytes)).map_err(|e| {
                                ApicentricError::runtime_error(
                                    format!("Failed to build proxy response: {}", e),
                                    None::<String>,
                                )
                            })?;
                            Self::record_log(
                                &state,
                                &service_name,
                                None,
                                method,
                                path,
                                status.as_u16(),
                                None,
                            )
                            .await;
                            Ok(final_resp)
                        }
                        Err(e) => {
                            let resp = Response::builder()
                                .status(StatusCode::BAD_GATEWAY)
                                .header("content-type", "application/json")
                                .body(Full::new(Bytes::from(format!(
                                    r#"{{"error": "Proxy request failed", "details": "{}"}}"#,
                                    e
                                ))))
                                .map_err(|e| {
                                    ApicentricError::runtime_error(
                                        format!("Failed to build proxy error response: {}", e),
                                        None::<String>,
                                    )
                                })?;
                            Self::record_log(
                                &state,
                                &service_name,
                                None,
                                method,
                                path,
                                StatusCode::BAD_GATEWAY.as_u16(),
                                None,
                            )
                            .await;
                            Ok(resp)
                        }
                    }
                } else if record_unknown {
                    let (placeholder_endpoint, recorded_path) =
                        ServiceInstance::build_recorded_endpoint(method, &relative_path);

                    let saved_definition = {
                        let mut def = definition.write().unwrap();
                        match def.endpoints.as_mut() {
                            Some(endpoints) => endpoints.push(placeholder_endpoint),
                            None => def.endpoints = Some(vec![placeholder_endpoint]),
                        }
                        def.clone()
                    };

                    if let Err(err) = storage.save_service(&saved_definition) {
                        log::warn!(
                            "Failed to persist recorded endpoint for {} {}: {}",
                            method,
                            relative_path,
                            err
                        );
                    }

                    let response_body = serde_json::json!({
                        "error": "Endpoint registrado",
                        "message": format!(
                            "Se registró automáticamente {} {} como {}",
                            method, path, recorded_path
                        ),
                        "recorded_path": recorded_path,
                        "method": method,
                    })
                    .to_string();

                    let mut response = Response::builder()
                        .status(StatusCode::CONFLICT)
                        .header("content-type", "application/json");

                    if let Some(cfg) = &cors_cfg {
                        let origin_hdr = headers.get("origin").cloned().unwrap_or_default();
                        let allow_origin = if cfg.origins.iter().any(|o| o == "*") {
                            "*".to_string()
                        } else if cfg
                            .origins
                            .iter()
                            .any(|o| o.eq_ignore_ascii_case(&origin_hdr))
                        {
                            origin_hdr.clone()
                        } else {
                            "*".to_string()
                        };
                        let allow_methods = cfg
                            .methods
                            .clone()
                            .map(|v| v.join(", "))
                            .unwrap_or_else(|| {
                                "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string()
                            });
                        let allow_headers = cfg
                            .headers
                            .clone()
                            .map(|v| v.join(", "))
                            .unwrap_or_else(|| "Content-Type, Authorization".to_string());

                        response = response
                            .header("access-control-allow-origin", &allow_origin)
                            .header("access-control-allow-methods", &allow_methods)
                            .header("access-control-allow-headers", &allow_headers);
                    } else {
                        response = response.header("access-control-allow-origin", "*");
                    }

                    let resp = response
                        .body(Full::new(Bytes::from(response_body)))
                        .map_err(|e| {
                            ApicentricError::runtime_error(
                                format!("Failed to build recorded response: {}", e),
                                None::<String>,
                            )
                        })?;
                    Self::record_log(
                        &state,
                        &service_name,
                        None,
                        method,
                        path,
                        StatusCode::CONFLICT.as_u16(),
                        None,
                    )
                    .await;
                    Ok(resp)
                } else {
                    let resp = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(format!(
                            r#"{{"error": "Endpoint not found", "method": "{}", "path": "{}", "service": "{}"}}"#,
                            method, relative_path, service_name
                        ))))
                        .map_err(|e| {
                            ApicentricError::runtime_error(
                                format!("Failed to build not found response: {}", e),
                                None::<String>,
                            )
                        })?;
                    Self::record_log(
                        &state,
                        &service_name,
                        None,
                        method,
                        path,
                        StatusCode::NOT_FOUND.as_u16(),
                        None,
                    )
                    .await;
                    Ok(resp)
                }
            }
        }
    }

    /// Internal helper to record a request log entry
    async fn record_log(
        state: &Arc<RwLock<ServiceState>>,
        service: &str,
        endpoint: Option<usize>,
        method: &str,
        path: &str,
        status: u16,
        payload: Option<String>,
    ) {
        let mut guard = state.write().await;
        guard.add_log_entry(RequestLogEntry::new(
            service.to_string(),
            endpoint,
            method.to_string(),
            path.to_string(),
            status,
            payload,
        ));
    }

    /// Execute a user-provided script with request context
    async fn execute_script(
        script_path: &Path,
        state: &Arc<RwLock<ServiceState>>,
        scripting_engine: &ScriptingEngine,
        path_params: &crate::simulator::service::routing::PathParameters,
        request_context: &RequestContext,
    ) -> ApicentricResult<Value> {
        let script_source = tokio::fs::read_to_string(script_path).await.map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to read script {}: {}", script_path.display(), e),
                Some("Check script path"),
            )
        })?;

        let state_guard = state.read().await;
        let context = serde_json::json!({
            "request": {
                "method": request_context.method.clone(),
                "path": request_context.path.clone(),
                "query": request_context.query.clone(),
                "headers": request_context.headers.clone(),
                "body": request_context.body.clone(),
            },
            "params": path_params.all().clone(),
            "fixtures": state_guard.all_fixtures().clone(),
            "runtime": state_guard.all_runtime_data().clone(),
        });
        drop(state_guard);

        let result = scripting_engine.execute(&script_source, &context)?;

        if let serde_json::Value::Object(ref map) = result {
            let mut state_guard = state.write().await;
            for (k, v) in map {
                state_guard.set_runtime_data(k.clone(), v.clone());
            }
        }

        Ok(result)
    }
}
