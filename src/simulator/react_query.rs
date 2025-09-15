use anyhow::Result;

use crate::simulator::config::{EndpointDefinition, EndpointKind, ServiceDefinition};

/// Generate React Query hooks for a service definition.
///
/// Each HTTP endpoint becomes either a `useQuery` (for GET) or `useMutation`
/// (for other methods) hook. The module only performs code generation,
/// keeping I/O concerns in higher layers.
pub fn to_react_query(service: &ServiceDefinition) -> Result<String> {
    let mut out = String::new();
    out.push_str("import { useQuery, useMutation } from '@tanstack/react-query';\n\n");

    for ep in &service.endpoints {
        if ep.kind != EndpointKind::Http {
            continue;
        }
        if ep.method.eq_ignore_ascii_case("GET") {
            out.push_str(&generate_query_hook(ep, &service.server.base_path));
        } else {
            out.push_str(&generate_mutation_hook(ep, &service.server.base_path));
        }
        out.push('\n');
    }

    Ok(out)
}

fn generate_query_hook(ep: &EndpointDefinition, base_path: &str) -> String {
    let hook_name = hook_name(ep, true);
    let params = path_params(&ep.path);
    let param_list = format_params(&params);
    let url = build_url(base_path, &ep.path, &params);
    format!(
        "export function {hook}(baseUrl: string{param_list}) {{\n    return useQuery(['{method}','{path}'], () => fetch(`${{baseUrl}}{url}`).then(res => res.json()));\n}}\n",
        hook = hook_name,
        method = ep.method.to_uppercase(),
        path = ep.path,
        url = url,
        param_list = param_list,
    )
}

fn generate_mutation_hook(ep: &EndpointDefinition, base_path: &str) -> String {
    let hook_name = hook_name(ep, false);
    let params = path_params(&ep.path);
    let param_list = format_params(&params);
    let url = build_url(base_path, &ep.path, &params);
    let method = ep.method.to_uppercase();
    format!(
        "export function {hook}(baseUrl: string{param_list}) {{\n    return useMutation((body: any) =>\n        fetch(`${{baseUrl}}{url}`, {{ method: '{method}', body: JSON.stringify(body) }}).then(res => res.json())\n    );\n}}\n",
        hook = hook_name,
        url = url,
        method = method,
        param_list = param_list,
    )
}

fn hook_name(ep: &EndpointDefinition, is_query: bool) -> String {
    let path_part = ep
        .path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_matches(|c| c == '{' || c == '}'))
        .map(capitalize)
        .collect::<String>();
    if is_query {
        format!("use{path_part}Query")
    } else {
        let method = capitalize(&ep.method.to_lowercase());
        format!("use{method}{path_part}Mutation")
    }
}

fn path_params(path: &str) -> Vec<String> {
    path
        .split('/')
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
            .map(|p| format!(", {p}: string"))
            .collect::<String>()
    }
}

fn build_url(base_path: &str, path: &str, params: &[String]) -> String {
    let mut full = format!("{}/{}", base_path.trim_end_matches('/'), path.trim_start_matches('/'));
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
    fn generates_query_and_mutation() {
        let service = ServiceDefinition {
            name: "Test".into(),
            version: None,
            description: None,
            server: ServerConfig { port: None, base_path: "/api".into(), proxy_base_url: None, cors: None },
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: vec![
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
            ],
            graphql: None,
            behavior: None,
        };
        let ts = to_react_query(&service).unwrap();
        assert!(ts.contains("usePetsQuery"));
        assert!(ts.contains("usePostPetsMutation"));
        assert!(ts.contains("/api/pets"));
    }
}

