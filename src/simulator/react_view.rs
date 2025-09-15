use crate::simulator::config::{EndpointDefinition, EndpointKind, ServiceDefinition};
use anyhow::Result;

/// Generate a minimal React view component for a service definition.
///
/// The generator follows Hexagonal architecture principles by staying pure and
/// delegating side effects to the caller. Given a `ServiceDefinition`, it
/// produces TSX that wires TanStack Query hooks with form elements so consumers
/// can quickly prototype interactions. Each HTTP endpoint becomes either a
/// readonly section (for queries) or a form (for mutations).
pub fn to_react_view(service: &ServiceDefinition) -> Result<String> {
    let mut out = String::new();
    out.push_str("import React, { useState } from 'react';\n");

    // Collect hooks required for this component
    let mut hooks = Vec::new();
    for ep in &service.endpoints {
        if ep.kind != EndpointKind::Http {
            continue;
        }
        let is_query = ep.method.eq_ignore_ascii_case("GET");
        let name = hook_name(ep, is_query);
        let params = path_params(&ep.path);
        hooks.push((name, params, is_query));
    }
    if !hooks.is_empty() {
        let imports = hooks
            .iter()
            .map(|(n, _, _)| n.clone())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("import {{ {imports} }} from './hooks';\n\n"));
    }

    out.push_str("export interface ServiceViewProps { baseUrl: string }\n");
    out.push_str("export function ServiceView({ baseUrl }: ServiceViewProps) {\n");

    // State for path params and bodies
    for (idx, (_, params, is_query)) in hooks.iter().enumerate() {
        for p in params {
            let cap = capitalize(p);
            out.push_str(&format!(
                "  const [{p}, set{cap}] = useState<string>('');\n",
                p = p,
                cap = cap
            ));
        }
        if !*is_query {
            out.push_str(&format!(
                "  const [body{idx}, setBody{idx}] = useState<string>('');\n",
                idx = idx
            ));
        }
    }

    // Invoke hooks
    for (idx, (name, params, is_query)) in hooks.iter().enumerate() {
        let mut args = vec!["baseUrl".to_string()];
        args.extend(params.iter().cloned());
        let arg_list = args.join(", ");
        if *is_query {
            out.push_str(&format!(
                "  const query{idx} = {name}({args});\n",
                idx = idx,
                name = name,
                args = arg_list
            ));
        } else {
            out.push_str(&format!(
                "  const mutation{idx} = {name}({args});\n",
                idx = idx,
                name = name,
                args = arg_list
            ));
        }
    }

    out.push_str("  return (\n    <div>\n");
    for (idx, (_, params, is_query)) in hooks.iter().enumerate() {
        if *is_query {
            out.push_str(&format!(
                "      <pre>{{JSON.stringify(query{idx}.data)}}</pre>\n",
                idx = idx
            ));
        } else {
            out.push_str(&format!(
                "      <form onSubmit={{e => {{ e.preventDefault(); mutation{idx}.mutate(body{idx}); }}}}>\n",
                idx = idx
            ));
            for p in params {
                let cap = capitalize(p);
                out.push_str(&format!(
                    "        <input value={{{p}}} onChange={{e => set{cap}(e.target.value)}} />\n",
                    p = p,
                    cap = cap
                ));
            }
            out.push_str(&format!(
                "        <input value={{body{idx}}} onChange={{e => setBody{idx}(e.target.value)}} />\n        <button type=\"submit\">Submit</button>\n      </form>\n",
                idx = idx
            ));
        }
    }
    out.push_str("    </div>\n  );\n}\n");

    Ok(out)
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
    use crate::simulator::config::{EndpointKind, ServerConfig};

    #[test]
    fn generates_view_with_query_and_mutation() {
        let service = ServiceDefinition {
            name: "Test".into(),
            version: None,
            description: None,
            server: ServerConfig {
                port: None,
                base_path: "/api".into(),
                proxy_base_url: None,
                cors: None,
            },
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
        let tsx = to_react_view(&service).unwrap();
        assert!(tsx.contains("ServiceView"));
        assert!(tsx.contains("usePetsQuery"));
        assert!(tsx.contains("usePostPetsMutation"));
    }
}
