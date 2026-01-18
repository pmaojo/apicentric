use anyhow::Result;

use crate::simulator::config::{EndpointDefinition, EndpointKind, ServiceDefinition};

/// Generate Axios client code for a service definition.
///
/// Each HTTP endpoint becomes a function that uses axios to make the request.
/// The module only performs code generation, keeping I/O concerns in higher layers.
pub fn to_axios_client(service: &ServiceDefinition) -> Result<String> {
    let mut out = String::new();
    out.push_str("import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';\n\n");

    // Generate the client class
    let class_name = format!("{}Client", capitalize(&service.name));
    out.push_str(&format!("export class {} {{\n", class_name));
    out.push_str("  private client: AxiosInstance;\n\n");

    // Constructor
    out.push_str("  constructor(baseURL: string, config?: AxiosRequestConfig) {\n");
    out.push_str("    this.client = axios.create({\n");
    out.push_str("      baseURL,\n");
    out.push_str("      ...config,\n");
    out.push_str("    });\n");
    out.push_str("  }\n\n");

    // Generate methods for each endpoint
    let endpoints = service.endpoints.as_ref().cloned().unwrap_or_default();
    let base_path = service.server.as_ref().map(|s| s.base_path.as_str()).unwrap_or("/");
    for ep in &endpoints {
        if ep.kind != EndpointKind::Http {
            continue;
        }
        out.push_str(&generate_client_method(ep, base_path));
        out.push('\n');
    }

    out.push_str("}\n");

    Ok(out)
}

fn generate_client_method(ep: &EndpointDefinition, base_path: &str) -> String {
    let method_name = method_name(ep);
    let params = path_params(&ep.path);
    let param_list = format_params(&params);
    let url = build_url(base_path, &ep.path, &params);
    let http_method = ep.method.to_lowercase();

    let mut method = format!("  async {}({}): Promise<any> {{\n", method_name, param_list);

    if http_method == "get" || http_method == "delete" {
        method.push_str(&format!(
            "    return this.client.{}(`{}`);\n",
            http_method, url
        ));
    } else {
        // POST, PUT, PATCH need a body parameter
        let body_param = if param_list.is_empty() {
            "data?: any".to_string()
        } else {
            ", data?: any".to_string()
        };
        method = format!(
            "  async {}({}{}): Promise<any> {{\n",
            method_name, param_list, body_param
        );
        method.push_str(&format!(
            "    return this.client.{}(`{}`, data);\n",
            http_method, url
        ));
    }

    method.push_str("  }\n");
    method
}

fn method_name(ep: &EndpointDefinition) -> String {
    let path_part = ep
        .path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_matches(|c| c == '{' || c == '}'))
        .map(capitalize)
        .collect::<String>();

    let method = ep.method.to_lowercase();
    format!("{}{}", method, path_part)
}

fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|seg| {
            if seg.starts_with('{') && seg.ends_with('}') {
                Some(seg[1..seg.len() - 1].to_string())
            } else {
                None
            }
        })
        .collect()
}

fn format_params(params: &[String]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        params
            .iter()
            .map(|p| format!("{}: string", p))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn build_url(base_path: &str, path: &str, params: &[String]) -> String {
    let mut full = format!(
        "{}/{}",
        base_path.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    for p in params {
        full = full.replace(&format!("{{{}}}", p), &format!("${{{}}}", p));
    }
    full
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::config::{EndpointDefinition, EndpointKind, ServerConfig};

    #[test]
    fn generates_axios_client() {
        let service = ServiceDefinition {
            name: "Pet".into(),
            version: None,
            description: None,
            server: Some(ServerConfig {
                port: None,
                base_path: "/api".into(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Some(vec![
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".into(),
                    path: "/pets".into(),
                    header_match: None,
                    description: None,
                    parameters: None,
                    request_body: None,
                    responses: Default::default(),
                    scenarios: None,
                    stream: None,
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "POST".into(),
                    path: "/pets".into(),
                    header_match: None,
                    description: None,
                    parameters: None,
                    request_body: None,
                    responses: Default::default(),
                    scenarios: None,
                    stream: None,
                },
                EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: "GET".into(),
                    path: "/pets/{id}".into(),
                    header_match: None,
                    description: None,
                    parameters: None,
                    request_body: None,
                    responses: Default::default(),
                    scenarios: None,
                    stream: None,
                },
            ]),
            graphql: None,
            behavior: None,
            twin: None,
        };
        let ts = to_axios_client(&service).unwrap();
        assert!(ts.contains("class PetClient"));
        assert!(ts.contains("async getPets()"));
        assert!(ts.contains("async postPets(data?: any)"));
        assert!(ts.contains("async getPetsId(id: string)"));
        assert!(ts.contains("/api/pets"));
    }
}
