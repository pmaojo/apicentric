use std::collections::{BTreeMap, HashMap, HashSet};

use openapi::{Info, Operation, Operations, Parameter, Response, Schema, Spec};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value;
use url::Url;

use crate::simulator::config::{
    EndpointDefinition, EndpointKind, ParameterDefinition, ParameterLocation,
    RequestBodyDefinition, ResponseDefinition, ServerConfig, ServiceDefinition,
};

/// Convert an OpenAPI spec into a `ServiceDefinition` used by the simulator
pub fn from_openapi(doc: &Value) -> ServiceDefinition {
    if doc.get("openapi").is_some() {
        return from_openapi_v3(doc);
    }

    match serde_yaml::from_value::<Spec>(doc.clone()) {
        Ok(spec) => from_openapi_v2(&spec, doc),
        Err(_) => from_openapi_v3(doc),
    }
}

fn from_openapi_v2(spec: &Spec, raw: &Value) -> ServiceDefinition {
    let json_doc = serde_json::to_value(raw).unwrap_or(JsonValue::Null);
    let raw_paths = json_doc.get("paths");

    let server = ServerConfig {
        port: None,
        base_path: spec.base_path.clone().unwrap_or_else(|| "/".to_string()),
        proxy_base_url: None,
        cors: None,
    };

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

    let mut endpoints = Vec::new();
    for (path, ops) in &spec.paths {
        let raw_path = raw_paths.and_then(|paths| paths.get(path));
        let methods: Vec<(&str, Option<&Operation>)> = vec![
            ("get", ops.get.as_ref()),
            ("post", ops.post.as_ref()),
            ("put", ops.put.as_ref()),
            ("patch", ops.patch.as_ref()),
            ("delete", ops.delete.as_ref()),
        ];

        for (method, op_opt) in methods {
            if let Some(op) = op_opt {
                let raw_operation = raw_path.and_then(|p| p.get(method));

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

                let mut responses_map = HashMap::new();
                let raw_responses = raw_operation.and_then(|op| op.get("responses"));
                for (status, resp) in &op.responses {
                    if let Ok(code) = status.parse::<u16>() {
                        let raw_response = raw_responses.and_then(|r| r.get(status));
                        let content_type = determine_content_type_v2(op, spec, raw_response);
                        let body = build_response_body_v2(
                            resp,
                            raw_response,
                            &content_type,
                            &spec.definitions,
                        );
                        responses_map.insert(
                            code,
                            ResponseDefinition {
                                condition: None,
                                content_type,
                                body,
                                script: None,
                                headers: None,
                                side_effects: None,
                            },
                        );
                    }
                }

                endpoints.push(EndpointDefinition {
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
                });
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

fn from_openapi_v3(raw: &Value) -> ServiceDefinition {
    match serde_yaml::from_value::<OpenApi3Document>(raw.clone()) {
        Ok(doc) => convert_openapi3(&doc),
        Err(_) => ServiceDefinition {
            name: raw
                .get("info")
                .and_then(|info| info.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string(),
            version: raw
                .get("info")
                .and_then(|info| info.get("version"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            description: None,
            server: ServerConfig {
                port: None,
                base_path: "/".to_string(),
                proxy_base_url: None,
                cors: None,
            },
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Vec::new(),
            graphql: None,
            behavior: None,
        },
    }
}

fn convert_openapi3(doc: &OpenApi3Document) -> ServiceDefinition {
    let empty_schemas = BTreeMap::new();
    let component_schemas = doc
        .components
        .as_ref()
        .map(|c| &c.schemas)
        .unwrap_or(&empty_schemas);

    let models = if component_schemas.is_empty() {
        None
    } else {
        Some(
            component_schemas
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    };

    let base_path = doc
        .servers
        .iter()
        .find_map(|server| Some(extract_base_path(&server.url)))
        .unwrap_or_else(|| "/".to_string());

    let server = ServerConfig {
        port: None,
        base_path,
        proxy_base_url: None,
        cors: None,
    };

    let mut endpoints = Vec::new();
    for (path, item) in &doc.paths {
        let methods: Vec<(&str, Option<&OpenApi3Operation>)> = vec![
            ("get", item.get.as_ref()),
            ("post", item.post.as_ref()),
            ("put", item.put.as_ref()),
            ("patch", item.patch.as_ref()),
            ("delete", item.delete.as_ref()),
        ];

        for (method, op_opt) in methods {
            if let Some(op) = op_opt {
                let parameters = if op.parameters.is_empty() {
                    None
                } else {
                    Some(
                        op.parameters
                            .iter()
                            .map(|param| convert_openapi3_parameter(param))
                            .collect(),
                    )
                };

                let request_body = op.request_body.as_ref().and_then(|body| {
                    pick_media_type(&body.content).map(|(content_type, media)| {
                        let schema_ref = media
                            .schema
                            .as_ref()
                            .and_then(|schema| schema.get("$ref"))
                            .and_then(|r| r.as_str())
                            .and_then(|r| r.split('/').last())
                            .map(|s| s.to_string());
                        RequestBodyDefinition {
                            required: body.required,
                            schema: schema_ref,
                            content_type: Some(content_type.to_string()),
                        }
                    })
                });

                let mut responses_map = HashMap::new();
                for (status, response) in &op.responses {
                    if let Ok(code) = status.parse::<u16>() {
                        let (content_type, example) =
                            extract_example_from_v3(response, component_schemas);
                        let body = example
                            .map(|value| format_json_value(&value))
                            .unwrap_or_else(|| {
                                response
                                    .description
                                    .clone()
                                    .unwrap_or_else(|| "".to_string())
                            });
                        responses_map.insert(
                            code,
                            ResponseDefinition {
                                condition: None,
                                content_type,
                                body,
                                script: None,
                                headers: None,
                                side_effects: None,
                            },
                        );
                    }
                }

                endpoints.push(EndpointDefinition {
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
                });
            }
        }
    }

    ServiceDefinition {
        name: doc.info.title.clone(),
        version: Some(doc.info.version.clone()),
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

fn determine_content_type_v2(
    op: &Operation,
    spec: &Spec,
    raw_response: Option<&JsonValue>,
) -> String {
    if let Some(content_type) = op
        .produces
        .as_ref()
        .and_then(|values| values.first().cloned())
    {
        return content_type;
    }

    if let Some(content_type) = spec
        .produces
        .as_ref()
        .and_then(|values| values.first().cloned())
    {
        return content_type;
    }

    if let Some(raw) = raw_response {
        if let Some(examples) = raw.get("examples").and_then(|v| v.as_object()) {
            if let Some((ctype, _)) = examples.iter().next() {
                return ctype.clone();
            }
        }
    }

    "application/json".to_string()
}

fn build_response_body_v2(
    resp: &Response,
    raw_response: Option<&JsonValue>,
    content_type: &str,
    definitions: &BTreeMap<String, Schema>,
) -> String {
    if let Some(example) = extract_example_from_v2(raw_response, content_type) {
        return format_json_value(&example);
    }

    if let Some(schema) = &resp.schema {
        let mut visited = HashSet::new();
        if let Some(value) = generate_example_from_schema_v2(schema, definitions, &mut visited) {
            return format_json_value(&value);
        }
    }

    resp.description.clone()
}

fn extract_example_from_v2(
    raw_response: Option<&JsonValue>,
    content_type: &str,
) -> Option<JsonValue> {
    let resp = raw_response?;
    if let Some(examples) = resp.get("examples").and_then(|v| v.as_object()) {
        if let Some(example) = examples.get(content_type) {
            return Some(example.clone());
        }
        if let Some(example) = examples.values().next() {
            return Some(example.clone());
        }
    }

    if let Some(example) = resp.get("example") {
        return Some(example.clone());
    }

    resp.get("schema")
        .and_then(|schema| schema.get("example").cloned())
}

fn generate_example_from_schema_v2(
    schema: &Schema,
    definitions: &BTreeMap<String, Schema>,
    visited: &mut HashSet<String>,
) -> Option<JsonValue> {
    if let Some(ref_path) = &schema.ref_path {
        if let Some(name) = ref_path.split('/').last() {
            if !visited.insert(name.to_string()) {
                return Some(JsonValue::Object(Default::default()));
            }
            let result = definitions
                .get(name)
                .and_then(|s| generate_example_from_schema_v2(s, definitions, visited));
            visited.remove(name);
            return result;
        }
    }

    if let Some(schema_type) = schema.schema_type.as_deref() {
        match schema_type {
            "object" => {
                let mut map = serde_json::Map::new();
                if let Some(props) = &schema.properties {
                    for (prop_name, prop_schema) in props {
                        if let Some(value) =
                            generate_example_from_schema_v2(prop_schema, definitions, visited)
                        {
                            map.insert(prop_name.clone(), value);
                        }
                    }
                }
                return Some(JsonValue::Object(map));
            }
            "array" => {
                if let Some(items) = schema.items.as_ref() {
                    if let Some(value) =
                        generate_example_from_schema_v2(items, definitions, visited)
                    {
                        return Some(JsonValue::Array(vec![value]));
                    }
                }
                return Some(JsonValue::Array(vec![]));
            }
            "integer" => {
                if let Some(values) = &schema.enum_values {
                    if let Some(first) = values.first() {
                        if let Ok(num) = first.parse::<i64>() {
                            return Some(JsonValue::from(num));
                        }
                    }
                }
                return Some(JsonValue::from(0));
            }
            "number" => {
                if let Some(values) = &schema.enum_values {
                    if let Some(first) = values.first() {
                        if let Ok(num) = first.parse::<f64>() {
                            if let Some(value) = serde_json::Number::from_f64(num) {
                                return Some(JsonValue::Number(value));
                            }
                        }
                    }
                }
                return serde_json::Number::from_f64(0.0).map(JsonValue::Number);
            }
            "boolean" => return Some(JsonValue::Bool(true)),
            "string" => {
                if let Some(values) = &schema.enum_values {
                    if let Some(first) = values.first() {
                        return Some(JsonValue::String(first.clone()));
                    }
                }
                if let Some(format) = schema.format.as_deref() {
                    let sample = match format {
                        "date-time" => "1970-01-01T00:00:00Z",
                        "date" => "1970-01-01",
                        "uuid" => "00000000-0000-0000-0000-000000000000",
                        _ => "string",
                    };
                    return Some(JsonValue::String(sample.to_string()));
                }
                return Some(JsonValue::String("string".into()));
            }
            _ => {}
        }
    }

    if let Some(props) = &schema.properties {
        let mut map = serde_json::Map::new();
        for (prop_name, prop_schema) in props {
            if let Some(value) = generate_example_from_schema_v2(prop_schema, definitions, visited)
            {
                map.insert(prop_name.clone(), value);
            }
        }
        return Some(JsonValue::Object(map));
    }

    Some(JsonValue::Null)
}

fn format_json_value(value: &JsonValue) -> String {
    match value {
        JsonValue::Object(_) | JsonValue::Array(_) => {
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        }
        JsonValue::String(_) => serde_json::to_string(value)
            .unwrap_or_else(|_| value.as_str().unwrap_or_default().to_string()),
        _ => value.to_string(),
    }
}

fn convert_openapi3_parameter(param: &OpenApi3Parameter) -> ParameterDefinition {
    let schema = param.schema.as_ref().or_else(|| {
        param.content.as_ref().and_then(|content| {
            pick_media_type(content).and_then(|(_, media)| media.schema.as_ref())
        })
    });

    ParameterDefinition {
        name: param.name.clone(),
        location: match param.location.as_str() {
            "path" => ParameterLocation::Path,
            "query" => ParameterLocation::Query,
            "header" => ParameterLocation::Header,
            _ => ParameterLocation::Header,
        },
        param_type: infer_schema_type(schema),
        required: param.required,
        description: param.description.clone(),
    }
}

fn infer_schema_type(schema: Option<&JsonValue>) -> String {
    if let Some(schema) = schema {
        if let Some(schema_type) = schema.get("type").and_then(|v| v.as_str()) {
            return schema_type.to_string();
        }
        if schema.get("$ref").is_some() {
            return "object".to_string();
        }
    }
    "string".to_string()
}

fn pick_media_type(
    content: &BTreeMap<String, OpenApi3MediaType>,
) -> Option<(&str, &OpenApi3MediaType)> {
    if let Some(media) = content.get("application/json") {
        return Some(("application/json", media));
    }
    content
        .iter()
        .next()
        .map(|(ctype, media)| (ctype.as_str(), media))
}

fn extract_example_from_v3(
    response: &OpenApi3Response,
    component_schemas: &BTreeMap<String, JsonValue>,
) -> (String, Option<JsonValue>) {
    if !response.content.is_empty() {
        if let Some((content_type, media)) = pick_media_type(&response.content) {
            if let Some(example) = media
                .example
                .clone()
                .or_else(|| media.examples.values().find_map(|ex| ex.value.clone()))
                .or_else(|| {
                    media
                        .schema
                        .as_ref()
                        .and_then(|schema| schema.get("example").cloned())
                })
            {
                return (content_type.to_string(), Some(example));
            }

            if let Some(schema) = &media.schema {
                let mut visited = HashSet::new();
                let value =
                    generate_example_from_schema_v3(schema, component_schemas, &mut visited);
                return (content_type.to_string(), Some(value));
            }

            return (content_type.to_string(), None);
        }
    }

    ("application/json".to_string(), None)
}

fn generate_example_from_schema_v3(
    schema: &JsonValue,
    component_schemas: &BTreeMap<String, JsonValue>,
    visited: &mut HashSet<String>,
) -> JsonValue {
    if let Some(example) = schema.get("example") {
        return example.clone();
    }

    if let Some(default_value) = schema.get("default") {
        return default_value.clone();
    }

    if let Some(ref_path) = schema.get("$ref").and_then(|r| r.as_str()) {
        if let Some(name) = ref_path.split('/').last() {
            if !visited.insert(name.to_string()) {
                return JsonValue::Object(Default::default());
            }
            let value = component_schemas
                .get(name)
                .map(|target| generate_example_from_schema_v3(target, component_schemas, visited))
                .unwrap_or_else(|| JsonValue::Object(Default::default()));
            visited.remove(name);
            return value;
        }
    }

    if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
        let mut map = serde_json::Map::new();
        for subschema in all_of {
            let value = generate_example_from_schema_v3(subschema, component_schemas, visited);
            if let JsonValue::Object(obj) = value {
                for (key, val) in obj {
                    map.insert(key, val);
                }
            }
        }
        return JsonValue::Object(map);
    }

    if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_array()) {
        if let Some(first) = one_of.first() {
            return generate_example_from_schema_v3(first, component_schemas, visited);
        }
    }

    if let Some(any_of) = schema.get("anyOf").and_then(|v| v.as_array()) {
        if let Some(first) = any_of.first() {
            return generate_example_from_schema_v3(first, component_schemas, visited);
        }
    }

    if let Some(schema_type) = schema.get("type").and_then(|v| v.as_str()) {
        match schema_type {
            "object" => {
                let mut map = serde_json::Map::new();
                if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                    for (name, prop_schema) in props {
                        let value = generate_example_from_schema_v3(
                            prop_schema,
                            component_schemas,
                            visited,
                        );
                        map.insert(name.clone(), value);
                    }
                }
                return JsonValue::Object(map);
            }
            "array" => {
                if let Some(items) = schema.get("items") {
                    let value = generate_example_from_schema_v3(items, component_schemas, visited);
                    return JsonValue::Array(vec![value]);
                }
                return JsonValue::Array(vec![]);
            }
            "integer" => {
                if let Some(values) = schema.get("enum").and_then(|v| v.as_array()) {
                    if let Some(first) = values.first() {
                        return first.clone();
                    }
                }
                return JsonValue::from(0);
            }
            "number" => {
                if let Some(values) = schema.get("enum").and_then(|v| v.as_array()) {
                    if let Some(first) = values.first() {
                        return first.clone();
                    }
                }
                return JsonValue::from(0.0);
            }
            "boolean" => return JsonValue::Bool(true),
            "string" => {
                if let Some(values) = schema.get("enum").and_then(|v| v.as_array()) {
                    if let Some(first) = values.first() {
                        return first.clone();
                    }
                }
                if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
                    let sample = match format {
                        "date-time" => "1970-01-01T00:00:00Z",
                        "date" => "1970-01-01",
                        "uuid" => "00000000-0000-0000-0000-000000000000",
                        "email" => "user@example.com",
                        _ => "string",
                    };
                    return JsonValue::String(sample.to_string());
                }
                return JsonValue::String("string".into());
            }
            _ => {}
        }
    }

    if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
        let mut map = serde_json::Map::new();
        for (name, prop_schema) in props {
            let value = generate_example_from_schema_v3(prop_schema, component_schemas, visited);
            map.insert(name.clone(), value);
        }
        return JsonValue::Object(map);
    }

    JsonValue::Null
}

fn extract_base_path(url: &str) -> String {
    if url.is_empty() {
        return "/".to_string();
    }

    if let Ok(parsed) = Url::parse(url) {
        let path = parsed.path();
        return if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        };
    }

    if let Some(idx) = url.find("//") {
        let remainder = &url[idx + 2..];
        if let Some(pos) = remainder.find('/') {
            let path = &remainder[pos..];
            return if path.is_empty() {
                "/".to_string()
            } else {
                path.to_string()
            };
        }
        return "/".to_string();
    }

    if url.starts_with('/') {
        return url.to_string();
    }

    if let Some(pos) = url.find('/') {
        let path = &url[pos..];
        return if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        };
    }

    "/".to_string()
}

#[derive(Debug, Deserialize)]
struct OpenApi3Document {
    pub info: OpenApi3Info,
    #[serde(default)]
    pub servers: Vec<OpenApi3Server>,
    pub paths: BTreeMap<String, OpenApi3PathItem>,
    #[serde(default)]
    pub components: Option<OpenApi3Components>,
}

#[derive(Debug, Deserialize)]
struct OpenApi3Info {
    pub title: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct OpenApi3Server {
    pub url: String,
}

#[derive(Debug, Deserialize, Default)]
struct OpenApi3PathItem {
    #[serde(default)]
    pub get: Option<OpenApi3Operation>,
    #[serde(default)]
    pub post: Option<OpenApi3Operation>,
    #[serde(default)]
    pub put: Option<OpenApi3Operation>,
    #[serde(default)]
    pub patch: Option<OpenApi3Operation>,
    #[serde(default)]
    pub delete: Option<OpenApi3Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenApi3Operation {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Vec<OpenApi3Parameter>,
    #[serde(default)]
    pub request_body: Option<OpenApi3RequestBody>,
    #[serde(default)]
    pub responses: BTreeMap<String, OpenApi3Response>,
}

#[derive(Debug, Deserialize)]
struct OpenApi3Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub schema: Option<JsonValue>,
    #[serde(default)]
    pub content: Option<BTreeMap<String, OpenApi3MediaType>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenApi3RequestBody {
    #[serde(default)]
    pub required: bool,
    pub content: BTreeMap<String, OpenApi3MediaType>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenApi3Response {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub content: BTreeMap<String, OpenApi3MediaType>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenApi3MediaType {
    #[serde(default)]
    pub schema: Option<JsonValue>,
    #[serde(default)]
    pub example: Option<JsonValue>,
    #[serde(default)]
    pub examples: BTreeMap<String, OpenApi3Example>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenApi3Example {
    #[serde(default)]
    pub value: Option<JsonValue>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenApi3Components {
    #[serde(default)]
    pub schemas: BTreeMap<String, JsonValue>,
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
