use std::collections::BTreeMap;
use std::collections::HashMap;

use openapi::{Info, Operation, Operations, Parameter, Response, Schema, Spec};

use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ParameterDefinition, ParameterLocation,
    RequestBodyDefinition, ResponseDefinition, ServerConfig, ServiceDefinition,
};

/// Convert an OpenAPI spec into a `ServiceDefinition` used by the simulator
pub fn from_openapi(spec: &Spec) -> ServiceDefinition {
    // server
    let server = ServerConfig {
        port: None,
        base_path: spec.base_path.clone().unwrap_or_else(|| "/".to_string()),
        proxy_base_url: None,
        cors: None,
    };

    // models
    let models = if spec.definitions.is_empty() {
        None
    } else {
        let mut map = HashMap::new();
        for (name, schema) in &spec.definitions {
            if let Ok(value) = serde_json::to_value(schema) {
                map.insert(name.clone(), value);
            }
        }
        Some(map)
    };

    // endpoints
    let mut endpoints = Vec::new();
    for (path, ops) in &spec.paths {
        let methods: Vec<(&str, Option<&Operation>)> = vec![
            ("get", ops.get.as_ref()),
            ("post", ops.post.as_ref()),
            ("put", ops.put.as_ref()),
            ("patch", ops.patch.as_ref()),
            ("delete", ops.delete.as_ref()),
        ];

        for (method, op_opt) in methods {
            if let Some(op) = op_opt {
                // parameters
                let mut params: Vec<ParameterDefinition> = Vec::new();
                let mut request_body: Option<RequestBodyDefinition> = None;
                if let Some(ps) = &op.parameters {
                    for p in ps {
                        if p.location == "body" {
                            request_body = Some(RequestBodyDefinition {
                                required: p.required.unwrap_or(false),
                                schema: p.schema.as_ref().and_then(|s| {
                                    s.ref_path
                                        .as_ref()
                                        .and_then(|r| r.split('/').last().map(|s| s.to_string()))
                                }),
                                content_type: op.consumes.as_ref().and_then(|c| c.first().cloned()),
                            });
                        } else {
                            params.push(ParameterDefinition {
                                name: p.name.clone(),
                                location: match p.location.as_str() {
                                    "path" => ParameterLocation::Path,
                                    "query" => ParameterLocation::Query,
                                    _ => ParameterLocation::Header,
                                },
                                param_type: p.param_type.clone().unwrap_or_else(|| "string".into()),
                                required: p.required.unwrap_or(false),
                                description: None,
                            });
                        }
                    }
                }

                let parameters = if params.is_empty() {
                    None
                } else {
                    Some(params)
                };

                // responses
                let mut responses_map = HashMap::new();
                for (status, resp) in &op.responses {
                    if let Ok(code) = status.parse::<u16>() {
                        responses_map.insert(
                            code,
                            ResponseDefinition {
                                condition: None,
                                content_type: op
                                    .produces
                                    .as_ref()
                                    .and_then(|p| p.first().cloned())
                                    .unwrap_or_else(|| "application/json".into()),
                                body: resp.description.clone(),
                                headers: None,
                                side_effects: None,
                            },
                        );
                    }
                }

                let endpoint = EndpointDefinition {
                    kind: EndpointKind::Http,
                    method: method.to_uppercase(),
                    path: path.clone(),
                    header_match: None,
                    description: op.summary.clone().or(op.description.clone()),
                    parameters,
                    request_body,
                    responses: responses_map,
                    scenarios: None,
                    stream: None,
                };
                endpoints.push(endpoint);
            }
        }
    }

    ServiceDefinition {
        name: spec.info.title.clone(),
        version: Some(spec.info.version.clone()),
        description: None,
        server,
        models,
        fixtures: None,
        bucket: None,
        endpoints,
        graphql: None,
        behavior: None,
    }
}

/// Convert a `ServiceDefinition` into an OpenAPI spec
pub fn to_openapi(service: &ServiceDefinition) -> Spec {
    let mut paths: BTreeMap<String, Operations> = BTreeMap::new();

    for ep in &service.endpoints {
        let operations = paths.entry(ep.path.clone()).or_insert_with(|| Operations {
            get: None,
            post: None,
            put: None,
            patch: None,
            delete: None,
            parameters: None,
        });

        let mut op = Operation {
            summary: ep.description.clone(),
            description: ep.description.clone(),
            consumes: ep
                .request_body
                .as_ref()
                .and_then(|b| b.content_type.clone().map(|c| vec![c])),
            produces: ep
                .responses
                .values()
                .next()
                .map(|r| vec![r.content_type.clone()])
                .or_else(|| Some(vec!["application/json".into()])),
            schemes: None,
            tags: None,
            operation_id: None,
            responses: BTreeMap::new(),
            parameters: None,
        };

        // parameters
        if let Some(params) = &ep.parameters {
            let mut vec = Vec::new();
            for p in params {
                vec.push(Parameter {
                    name: p.name.clone(),
                    location: match p.location {
                        ParameterLocation::Path => "path".into(),
                        ParameterLocation::Query => "query".into(),
                        ParameterLocation::Header => "header".into(),
                    },
                    required: Some(p.required),
                    schema: None,
                    unique_items: None,
                    param_type: Some(p.param_type.clone()),
                    format: None,
                });
            }
            op.parameters = Some(vec);
        }

        if let Some(body) = &ep.request_body {
            let schema = body.schema.as_ref().map(|s| Schema {
                ref_path: Some(format!("#/definitions/{}", s)),
                description: None,
                schema_type: None,
                format: None,
                enum_values: None,
                required: None,
                items: None,
                properties: None,
            });
            let param = Parameter {
                name: "body".into(),
                location: "body".into(),
                required: Some(body.required),
                schema,
                unique_items: None,
                param_type: None,
                format: None,
            };
            match op.parameters {
                Some(ref mut vec) => vec.push(param),
                None => op.parameters = Some(vec![param]),
            }
        }

        for (code, resp) in &ep.responses {
            op.responses.insert(
                code.to_string(),
                Response {
                    description: resp.body.clone(),
                    schema: None,
                },
            );
        }

        match ep.method.to_lowercase().as_str() {
            "get" => operations.get = Some(op),
            "post" => operations.post = Some(op),
            "put" => operations.put = Some(op),
            "patch" => operations.patch = Some(op),
            "delete" => operations.delete = Some(op),
            _ => {}
        }
    }

    let mut definitions = BTreeMap::new();
    if let Some(models) = &service.models {
        for (name, value) in models {
            if let Ok(schema) = serde_json::from_value::<Schema>(value.clone()) {
                definitions.insert(name.clone(), schema);
            }
        }
    }

    Spec {
        swagger: "2.0".into(),
        info: Info {
            title: service.name.clone(),
            version: service.version.clone().unwrap_or_else(|| "1.0".into()),
            terms_of_service: None,
        },
        paths,
        definitions,
        schemes: None,
        host: None,
        base_path: Some(service.server.base_path.clone()),
        consumes: None,
        produces: None,
        parameters: None,
        responses: None,
        security_definitions: None,
        tags: None,
    }
}
