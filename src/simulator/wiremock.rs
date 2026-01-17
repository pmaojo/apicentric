use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ScenarioConditions, ScenarioDefinition,
    ScenarioResponse, ScenarioStrategy, ServerConfig, ServiceDefinition,
};

#[derive(Debug, Deserialize, Default)]
struct WiremockFile {
    #[serde(default)]
    mappings: Vec<WiremockStub>,
    #[serde(default)]
    meta: Option<WiremockMeta>,
}

#[derive(Debug, Deserialize, Default)]
struct WiremockMeta {
    #[serde(rename = "serverName")]
    server_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct WiremockStub {
    #[serde(default)]
    name: Option<String>,
    request: WiremockRequest,
    #[serde(default)]
    response: Option<WiremockResponse>,
    #[serde(default)]
    responses: Vec<WiremockResponse>,
}

#[derive(Debug, Deserialize, Clone)]
struct WiremockRequest {
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(rename = "urlPath", default)]
    url_path: Option<String>,
    #[serde(rename = "urlPattern", default)]
    url_pattern: Option<String>,
    #[serde(rename = "urlPathPattern", default)]
    url_path_pattern: Option<String>,
    #[serde(default)]
    headers: HashMap<String, WiremockValuePattern>,
    #[serde(rename = "bodyPatterns", default)]
    body_patterns: Vec<WiremockBodyPattern>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct WiremockValuePattern {
    #[serde(rename = "equalTo")]
    equal_to: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct WiremockBodyPattern {
    #[serde(rename = "equalToJson")]
    equal_to_json: Option<Value>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct WiremockResponse {
    #[serde(default)]
    status: Option<u16>,
    #[serde(default)]
    body: Option<String>,
    #[serde(rename = "jsonBody", default)]
    json_body: Option<Value>,
    #[serde(default)]
    headers: HashMap<String, String>,
}

/// Parse a WireMock stub mapping from a `serde_json::Value` into a [`ServiceDefinition`]
pub fn from_json(value: &Value) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string(value)?;
    from_str(&json_str)
}

/// Parse a WireMock stub mapping into a [`ServiceDefinition`]
pub fn from_str(json: &str) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let (stubs, meta) = parse_wiremock(json)?;
    if stubs.is_empty() {
        return Err("No WireMock mappings found".into());
    }
    Ok(convert(&stubs, meta.as_ref()))
}

/// Load a WireMock stub mapping file and convert it into a [`ServiceDefinition`]
pub fn from_path<P: AsRef<Path>>(path: P) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    from_str(&content)
}

fn parse_wiremock(
    json: &str,
) -> Result<(Vec<WiremockStub>, Option<WiremockMeta>), Box<dyn std::error::Error>> {
    if let Ok(file) = serde_json::from_str::<WiremockFile>(json) {
        if !file.mappings.is_empty() {
            return Ok((file.mappings, file.meta));
        }
    }
    if let Ok(stub) = serde_json::from_str::<WiremockStub>(json) {
        return Ok((vec![stub], None));
    }
    if let Ok(stubs) = serde_json::from_str::<Vec<WiremockStub>>(json) {
        if !stubs.is_empty() {
            return Ok((stubs, None));
        }
    }
    Err("Unsupported WireMock export format".into())
}

fn convert(stubs: &[WiremockStub], meta: Option<&WiremockMeta>) -> ServiceDefinition {
    let name = meta
        .and_then(|m| m.server_name.clone())
        .or_else(|| stubs.iter().find_map(|s| s.name.clone()))
        .unwrap_or_else(|| "WireMock Service".to_string());

    let endpoints = stubs.iter().map(convert_stub).collect();

    ServiceDefinition {
        name,
        version: None,
        description: None,
        server: ServerConfig {
            port: None,
            base_path: "/".into(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        },
        models: None,
        fixtures: None,
        bucket: None,
        endpoints,
        graphql: None,
        behavior: None,
        #[cfg(feature = "iot")]
        twin: None,
    }
}

fn convert_stub(stub: &WiremockStub) -> EndpointDefinition {
    let method = stub
        .request
        .method
        .as_deref()
        .unwrap_or("GET")
        .to_uppercase();
    let path = normalize_path(&stub.request);
    let header_match = convert_headers(&stub.request.headers);
    let body_conditions = convert_body_patterns(&stub.request.body_patterns);

    let mut responses = HashMap::new();
    let mut scenarios = Vec::new();

    if !stub.responses.is_empty() {
        for (idx, resp) in stub.responses.iter().enumerate() {
            let (status, definition) = convert_response(resp);
            scenarios.push(ScenarioDefinition {
                name: if idx == 0 { stub.name.clone() } else { None },
                conditions: None,
                response: ScenarioResponse { status, definition },
                strategy: if idx == 0 {
                    Some(ScenarioStrategy::Sequential)
                } else {
                    None
                },
            });
        }
        responses.insert(404, empty_not_found_response());
    } else if let Some(resp) = stub.response.as_ref() {
        let (status, definition) = convert_response(resp);
        if let Some(body) = body_conditions {
            scenarios.push(ScenarioDefinition {
                name: stub.name.clone(),
                conditions: Some(ScenarioConditions {
                    query: None,
                    headers: None,
                    body: Some(body),
                }),
                response: ScenarioResponse { status, definition },
                strategy: None,
            });
            responses.insert(404, empty_body_mismatch_response());
        } else {
            responses.insert(status, definition);
        }
    } else {
        responses.insert(200, empty_default_response());
    }

    EndpointDefinition {
        kind: EndpointKind::Http,
        method,
        path,
        header_match,
        description: stub.name.clone(),
        parameters: None,
        request_body: None,
        responses,
        scenarios: if scenarios.is_empty() {
            None
        } else {
            Some(scenarios)
        },
        stream: None,
    }
}

fn normalize_path(request: &WiremockRequest) -> String {
    if let Some(url) = &request.url {
        if let Ok(parsed) = url::Url::parse(url) {
            let path = parsed.path();
            return if path.is_empty() {
                "/".into()
            } else {
                path.to_string()
            };
        }
        return ensure_leading_slash(url);
    }
    if let Some(url_path) = &request.url_path {
        return ensure_leading_slash(url_path);
    }
    if let Some(pattern) = &request.url_path_pattern {
        return ensure_leading_slash(pattern);
    }
    if let Some(pattern) = &request.url_pattern {
        return ensure_leading_slash(pattern);
    }
    "/".into()
}

fn ensure_leading_slash(value: &str) -> String {
    if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{}", value)
    }
}

fn convert_headers(
    headers: &HashMap<String, WiremockValuePattern>,
) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (name, pattern) in headers {
        if let Some(value) = &pattern.equal_to {
            map.insert(name.clone(), value.clone());
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

fn convert_body_patterns(patterns: &[WiremockBodyPattern]) -> Option<HashMap<String, Value>> {
    let mut map = HashMap::new();
    for pattern in patterns {
        if let Some(Value::Object(obj)) = &pattern.equal_to_json {
            for (key, value) in obj {
                map.insert(key.clone(), value.clone());
            }
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

fn convert_response(response: &WiremockResponse) -> (u16, ResponseDefinition) {
    let status = response.status.unwrap_or(200);
    let mut headers = response.headers.clone();
    let body = if let Some(json_body) = &response.json_body {
        if !headers.contains_key("Content-Type") {
            headers.insert("Content-Type".into(), "application/json".into());
        }
        serde_json::to_string(json_body).unwrap_or_else(|_| json_body.to_string())
    } else {
        response.body.clone().unwrap_or_default()
    };

    let content_type = headers.get("Content-Type").cloned().unwrap_or_else(|| {
        if response.json_body.is_some() {
            "application/json".into()
        } else {
            "text/plain".into()
        }
    });

    let headers_opt = if headers.is_empty() {
        None
    } else {
        Some(headers)
    };

    (
        status,
        ResponseDefinition {
            condition: None,
            content_type,
            body,
            script: None,
            headers: headers_opt,
            side_effects: None,
        },
    )
}

fn empty_default_response() -> ResponseDefinition {
    ResponseDefinition {
        condition: None,
        content_type: "application/json".into(),
        body: "{}".into(),
        script: None,
        headers: None,
        side_effects: None,
    }
}

fn empty_not_found_response() -> ResponseDefinition {
    ResponseDefinition {
        condition: None,
        content_type: "application/json".into(),
        body: "{\"error\":\"No matching WireMock scenario\"}".into(),
        script: None,
        headers: None,
        side_effects: None,
    }
}

fn empty_body_mismatch_response() -> ResponseDefinition {
    ResponseDefinition {
        condition: None,
        content_type: "application/json".into(),
        body: "{\"error\":\"No WireMock body pattern matched\"}".into(),
        script: None,
        headers: None,
        side_effects: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_stub_with_headers_and_body_patterns() {
        let json = r#"{
            "meta": {"serverName": "Payments"},
            "mappings": [
                {
                    "name": "capture payment",
                    "request": {
                        "method": "post",
                        "url": "/payments",
                        "headers": {
                            "Content-Type": {"equalTo": "application/json"}
                        },
                        "bodyPatterns": [
                            {"equalToJson": {"kind": "capture", "amount": 42}}
                        ]
                    },
                    "response": {
                        "status": 201,
                        "jsonBody": {"id": "pmt_123", "status": "captured"},
                        "headers": {"Content-Type": "application/json"}
                    }
                }
            ]
        }"#;

        let service = from_str(json).expect("wiremock conversion should succeed");
        assert_eq!(service.name, "Payments");
        assert_eq!(service.endpoints.len(), 1);
        let endpoint = &service.endpoints[0];
        assert_eq!(endpoint.method, "POST");
        assert_eq!(endpoint.path, "/payments");
        let header_match = endpoint.header_match.as_ref().expect("header match");
        assert_eq!(
            header_match.get("Content-Type").map(String::as_str),
            Some("application/json")
        );
        let scenarios = endpoint
            .scenarios
            .as_ref()
            .expect("scenario for body pattern");
        assert_eq!(scenarios.len(), 1);
        let scenario = &scenarios[0];
        let body_conditions = scenario
            .conditions
            .as_ref()
            .expect("body condition")
            .body
            .as_ref()
            .expect("body map");
        assert_eq!(
            body_conditions.get("kind"),
            Some(&serde_json::json!("capture"))
        );
        assert_eq!(body_conditions.get("amount"), Some(&serde_json::json!(42)));
        let response = &scenario.response;
        assert_eq!(response.status, 201);
        assert!(response.definition.body.contains("\"status\":\"captured\""));
    }

    #[test]
    fn parses_multiple_responses_into_sequential_scenarios() {
        let json = r#"{
            "mappings": [
                {
                    "request": {"method": "get", "url": "/status"},
                    "responses": [
                        {"status": 200, "body": "OK"},
                        {"status": 500, "body": "FAIL"}
                    ]
                }
            ]
        }"#;

        let service = from_str(json).expect("wiremock conversion should succeed");
        assert_eq!(service.name, "WireMock Service");
        let endpoint = &service.endpoints[0];
        let scenarios = endpoint.scenarios.as_ref().expect("sequential scenarios");
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].response.status, 200);
        assert!(matches!(
            scenarios[0].strategy,
            Some(crate::simulator::config::ScenarioStrategy::Sequential)
        ));
        assert_eq!(scenarios[1].response.status, 500);
        assert!(endpoint.responses.contains_key(&404));
    }
}
