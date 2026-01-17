use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
};

#[derive(Debug, Deserialize)]
struct MockoonHeader {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct MockoonResponse {
    #[serde(rename = "statusCode")]
    status_code: u16,
    body: String,
    #[serde(rename = "contentType", default)]
    content_type: Option<String>,
    #[serde(default)]
    headers: Vec<MockoonHeader>,
}

#[derive(Debug, Deserialize)]
struct MockoonRoute {
    method: String,
    endpoint: String,
    #[serde(default)]
    responses: Vec<MockoonResponse>,
}

#[derive(Debug, Deserialize)]
struct MockoonEnvironment {
    name: String,
    port: u16,
    #[serde(rename = "endpointPrefix", default)]
    endpoint_prefix: String,
    #[serde(default)]
    routes: Vec<MockoonRoute>,
}

fn convert(env: &MockoonEnvironment) -> ServiceDefinition {
    let base_path = if env.endpoint_prefix.is_empty() {
        "/".to_string()
    } else if env.endpoint_prefix.starts_with('/') {
        env.endpoint_prefix.clone()
    } else {
        format!("/{}", env.endpoint_prefix)
    };

    let server = ServerConfig {
        port: Some(env.port),
        base_path,
        proxy_base_url: None,
        cors: None,
        record_unknown: false,
    };

    let endpoints = env
        .routes
        .iter()
        .map(|r| {
            let mut responses = HashMap::new();
            for resp in &r.responses {
                let mut headers = HashMap::new();
                for h in &resp.headers {
                    headers.insert(h.key.clone(), h.value.clone());
                }
                let response = ResponseDefinition {
                    condition: None,
                    content_type: resp
                        .content_type
                        .clone()
                        .unwrap_or_else(|| "application/json".into()),
                    body: resp.body.clone(),
                    script: None,
                    headers: if headers.is_empty() {
                        None
                    } else {
                        Some(headers)
                    },
                    side_effects: None,
                };
                responses.insert(resp.status_code, response);
            }
            EndpointDefinition {
                kind: EndpointKind::Http,
                method: r.method.to_uppercase(),
                path: if r.endpoint.starts_with('/') {
                    r.endpoint.clone()
                } else {
                    format!("/{}", r.endpoint)
                },
                header_match: None,
                description: None,
                parameters: None,
                request_body: None,
                responses,
                scenarios: None,
                stream: None,
            }
        })
        .collect();

    ServiceDefinition {
        name: env.name.clone(),
        version: None,
        description: None,
        server,
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

fn parse_env(json: &str) -> Result<MockoonEnvironment, Box<dyn std::error::Error>> {
    if let Ok(env) = serde_json::from_str::<MockoonEnvironment>(json) {
        return Ok(env);
    }
    let envs: Vec<MockoonEnvironment> = serde_json::from_str(json)?;
    envs.into_iter()
        .next()
        .ok_or_else(|| "No environments found".into())
}

/// Parse a Mockoon JSON export from a `serde_json::Value` into a [`ServiceDefinition`]
pub fn from_json(
    value: &serde_json::Value,
) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string(value)?;
    from_str(&json_str)
}

/// Parse a Mockoon JSON export into a [`ServiceDefinition`]
pub fn from_str(json: &str) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let env = parse_env(json)?;
    Ok(convert(&env))
}

/// Load a Mockoon JSON file and convert it into a [`ServiceDefinition`]
pub fn from_path<P: AsRef<Path>>(path: P) -> Result<ServiceDefinition, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    from_str(&content)
}
