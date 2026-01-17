use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
};

/// Parse a Postman or Insomnia collection from a `serde_json::Value` into a [`ServiceDefinition`]
pub fn from_json(v: &Value) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    if v.get("item").is_some() {
        Ok(convert_postman(v))
    } else if v.get("resources").is_some() {
        Ok(convert_insomnia(v))
    } else {
        Err("Unsupported collection format: Not a valid Postman or Insomnia export.".into())
    }
}

/// Parse a Postman or Insomnia collection from a string into a [`ServiceDefinition`]
pub fn from_str(json_str: &str) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let v: Value = serde_json::from_str(json_str)?;
    from_json(&v)
}

/// Load a Postman/Insomnia collection from a file path
pub fn from_path<P: AsRef<Path>>(path: P) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    from_str(&content)
}

fn convert_postman(v: &Value) -> ServiceDefinition {
    let name = v
        .get("info")
        .and_then(|i| i.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Postman Service")
        .to_string();

    let mut endpoints = Vec::new();
    if let Some(items) = v.get("item").and_then(|i| i.as_array()) {
        collect_postman_items(items, &mut endpoints);
    }

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

fn collect_postman_items(items: &[Value], endpoints: &mut Vec<EndpointDefinition>) {
    for item in items {
        if let Some(req) = item.get("request") {
            let desc = item
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            let method = req
                .get("method")
                .and_then(|m| m.as_str())
                .unwrap_or("GET")
                .to_uppercase();
            let path = extract_postman_path(req.get("url"));

            let mut responses = HashMap::new();
            if let Some(resp_arr) = item.get("response").and_then(|r| r.as_array()) {
                for resp in resp_arr {
                    let code = resp.get("code").and_then(|c| c.as_u64()).unwrap_or(200) as u16;
                    let body = resp
                        .get("body")
                        .and_then(|b| b.as_str())
                        .unwrap_or("")
                        .to_string();
                    let headers_map = resp.get("header").and_then(|h| h.as_array()).map(|arr| {
                        arr.iter()
                            .filter_map(|h| {
                                let key = h.get("key")?.as_str()?;
                                let val = h.get("value").and_then(|v| v.as_str()).unwrap_or("");
                                Some((key.to_string(), val.to_string()))
                            })
                            .collect::<HashMap<_, _>>()
                    });
                    let headers = match headers_map {
                        Some(m) if !m.is_empty() => Some(m),
                        _ => None,
                    };
                    let content_type = headers
                        .as_ref()
                        .and_then(|m| m.get("Content-Type").cloned())
                        .unwrap_or_else(|| "application/json".into());
                    responses.insert(
                        code,
                        ResponseDefinition {
                            condition: None,
                            content_type,
                            body,
                            script: None,
                            headers,
                            side_effects: None,
                        },
                    );
                }
            }
            if responses.is_empty() {
                responses.insert(
                    200,
                    ResponseDefinition {
                        condition: None,
                        content_type: "application/json".into(),
                        body: String::new(),
                        script: None,
                        headers: None,
                        side_effects: None,
                    },
                );
            }

            endpoints.push(EndpointDefinition {
                kind: EndpointKind::Http,
                method,
                path,
                header_match: None,
                description: desc,
                parameters: None,
                request_body: None,
                responses,
                scenarios: None,
                stream: None,
            });
        } else if let Some(sub) = item.get("item").and_then(|i| i.as_array()) {
            collect_postman_items(sub, endpoints);
        }
    }
}

fn extract_postman_path(url_val: Option<&Value>) -> String {
    if let Some(url) = url_val {
        if let Some(raw) = url.get("raw").and_then(|r| r.as_str()) {
            if let Ok(u) = url::Url::parse(raw) {
                u.path().to_string()
            } else if raw.starts_with('/') {
                raw.to_string()
            } else {
                format!("/{}", raw)
            }
        } else if let Some(path_arr) = url.get("path").and_then(|p| p.as_array()) {
            let parts: Vec<String> = path_arr
                .iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect();
            format!("/{}", parts.join("/"))
        } else {
            "/".into()
        }
    } else {
        "/".into()
    }
}

fn convert_insomnia(v: &Value) -> ServiceDefinition {
    let name = v
        .get("resources")
        .and_then(|r| r.as_array())
        .and_then(|arr| {
            arr.iter().find_map(|res| {
                if res.get("_type").and_then(|t| t.as_str()) == Some("workspace") {
                    res.get("name").and_then(|n| n.as_str())
                } else {
                    None
                }
            })
        })
        .unwrap_or("Insomnia Service")
        .to_string();

    let mut endpoints = Vec::new();
    if let Some(resources) = v.get("resources").and_then(|r| r.as_array()) {
        for res in resources {
            if res.get("_type").and_then(|t| t.as_str()) == Some("request") {
                let method = res
                    .get("method")
                    .and_then(|m| m.as_str())
                    .unwrap_or("GET")
                    .to_uppercase();
                let url_raw = res.get("url").and_then(|u| u.as_str()).unwrap_or("/");
                let path = if let Ok(u) = url::Url::parse(url_raw) {
                    u.path().to_string()
                } else if url_raw.starts_with('/') {
                    url_raw.to_string()
                } else {
                    format!("/{}", url_raw)
                };
                let desc = res
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());
                let mut responses = HashMap::new();
                responses.insert(
                    200,
                    ResponseDefinition {
                        condition: None,
                        content_type: "application/json".into(),
                        body: String::new(),
                        script: None,
                        headers: None,
                        side_effects: None,
                    },
                );
                endpoints.push(EndpointDefinition {
                    kind: EndpointKind::Http,
                    method,
                    path,
                    header_match: None,
                    description: desc,
                    parameters: None,
                    request_body: None,
                    responses,
                    scenarios: None,
                    stream: None,
                });
            }
        }
    }

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

/// Convert a [`ServiceDefinition`] into a Postman collection value
pub fn to_postman(service: &ServiceDefinition) -> Value {
    let items: Vec<Value> = service
        .endpoints
        .iter()
        .map(|ep| {
            let responses: Vec<Value> = ep
                .responses
                .iter()
                .map(|(status, resp)| {
                    let headers_vec: Vec<Value> = resp
                        .headers
                        .as_ref()
                        .map(|h| {
                            h.iter()
                                .map(|(k, v)| json!({ "key": k, "value": v }))
                                .collect()
                        })
                        .unwrap_or_else(Vec::new);
                    json!({
                        "name": status.to_string(),
                        "originalRequest": {
                            "method": ep.method.clone(),
                            "header": [],
                            "url": { "raw": ep.path.clone() }
                        },
                        "status": status.to_string(),
                        "code": status,
                        "body": resp.body,
                        "header": headers_vec
                    })
                })
                .collect();

            json!({
                "name": ep
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("{} {}", ep.method, ep.path)),
                "request": {
                    "method": ep.method.clone(),
                    "header": [],
                    "url": { "raw": ep.path.clone() }
                },
                "response": responses
            })
        })
        .collect();

    json!({
        "info": {
            "name": service.name.clone(),
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items
    })
}

/// Serialize a [`ServiceDefinition`] into a Postman collection string
pub fn to_string(service: &ServiceDefinition) -> Result<String, Box<dyn std::error::Error>> {
    let val = to_postman(service);
    Ok(serde_json::to_string_pretty(&val)?)
}
