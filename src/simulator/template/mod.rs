//! Template engine for dynamic response generation
//!
//! This module provides a comprehensive template rendering system using Handlebars
//! that supports dynamic responses based on request data, fixtures, and service state.

use crate::errors::{PulseError, PulseResult};
use crate::simulator::service::{PathParameters, ServiceState};
use handlebars::Handlebars;
use serde_json::{Map, Value};
use std::collections::HashMap;

mod helpers;
use self::helpers::{
    default_helper, filter_helper, find_by_field_helper, find_by_multi_field_helper, find_helper,
    json_helper, length_helper, lower_helper, merge_helper, not_helper, now_helper, random_helper,
    random_string_helper, select_helper, upper_helper,
    and_helper, or_helper, eq_helper, ne_helper, gt_helper, gte_helper, lt_helper, lte_helper,
    contains_helper, starts_with_helper, ends_with_helper, regex_match_helper, exists_helper,
};

/// Template engine for rendering dynamic responses
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

/// Template context containing all available data for rendering
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub fixtures: HashMap<String, Value>,
    pub params: HashMap<String, String>,
    pub runtime: HashMap<String, Value>,
    pub request: RequestContext,
}

/// Request context information available in templates
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
}

impl TemplateEngine {
    /// Create a new template engine with built-in helpers
    pub fn new() -> PulseResult<Self> {
        let mut handlebars = Handlebars::new();

        // Register built-in helpers
        Self::register_helpers(&mut handlebars)?;

        Ok(Self { handlebars })
    }

    /// Pre-process template to convert pipe syntax to Handlebars helpers
    fn preprocess_template(&self, template: &str) -> String {
        use regex::Regex;

        // Handle simple cases first - just return fixtures as JSON
        let simple_fixture_regex = Regex::new(r"\{\{\s*fixtures\.(\w+)\s*\}\}").unwrap();
        let mut result = simple_fixture_regex
            .replace_all(template, "{{json fixtures.$1}}")
            .to_string();

        // Handle complex pipe operations with multiple pipes
        let complex_pipe_regex = Regex::new(r"\{\{\s*([^{}]+?)\s*\}\}").unwrap();

        result = complex_pipe_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let content = caps.get(1).unwrap().as_str().trim();

                // Skip if it's already processed (contains json, length, etc.)
                if content.starts_with("json ")
                    || content.starts_with("length ")
                    || content.starts_with("find_by_field ")
                    || content.starts_with("merge ")
                {
                    return format!("{{{{{}}}}}", content);
                }

                // Handle 'not (...)' expressions
                if content.starts_with("not (") && content.ends_with(")") {
                    let inner_content = &content[5..content.len() - 1]; // Remove "not (" and ")"
                    if inner_content.contains(" | ") {
                        // Process the inner pipe expression first
                        let inner_processed = Self::process_pipe_expression(inner_content);
                        return format!("{{{{not ({})}}}}", inner_processed);
                    } else {
                        return format!("{{{{not {}}}}}", inner_content);
                    }
                }

                // Parse pipe operations
                if content.contains(" | ") {
                    let processed = Self::process_pipe_expression(content);
                    return format!("{{{{{}}}}}", processed);
                }

                // Handle simple non-piped expressions
                Self::process_simple_expression(content)
            })
            .to_string();

        result
    }

    /// Process pipe expressions like "fixtures.users | find(id=params.id)"
    fn process_pipe_expression(content: &str) -> String {
        let parts: Vec<&str> = content.split(" | ").collect();
        if parts.len() >= 2 {
            let mut current_value = parts[0].trim().to_string();

            for pipe_part in &parts[1..] {
                let pipe_part = pipe_part.trim();

                if pipe_part.starts_with("find(") && pipe_part.ends_with(")") {
                    // Parse find arguments: find(id=params.id) or find(email=request.body.email, password=request.body.password)
                    let args_content = &pipe_part[5..pipe_part.len() - 1]; // Remove "find(" and ")"
                    let conditions: Vec<&str> = args_content.split(',').map(|s| s.trim()).collect();

                    if conditions.len() == 1 {
                        // Single condition: find(id=params.id)
                        if let Some((key, val)) = conditions[0].split_once('=') {
                            current_value = format!(
                                "find_by_field {} \"{}\" {}",
                                current_value,
                                key.trim(),
                                val.trim()
                            );
                        }
                    } else {
                        // Multiple conditions: find(email=request.body.email, password=request.body.password)
                        let mut find_args = vec![current_value];
                        for condition in conditions {
                            if let Some((key, val)) = condition.split_once('=') {
                                find_args.push(format!("\"{}\"", key.trim()));
                                find_args.push(val.trim().to_string());
                            }
                        }
                        current_value = format!("find_by_multi_field {}", find_args.join(" "));
                    }
                } else if pipe_part.starts_with("select(") && pipe_part.ends_with(")") {
                    // Parse select arguments: select("id", "email") -> select current_value "id" "email"
                    let args_content = &pipe_part[7..pipe_part.len() - 1]; // Remove "select(" and ")"
                    let fields: Vec<&str> = args_content.split(',').map(|s| s.trim()).collect();
                    let field_args = fields.join(" ");
                    current_value = format!("select ({}) {}", current_value, field_args);
                } else if pipe_part == "length" {
                    current_value = format!("length {}", current_value);
                } else if pipe_part.starts_with("merge(") && pipe_part.ends_with(")") {
                    let args_content = &pipe_part[6..pipe_part.len() - 1]; // Remove "merge(" and ")"
                    current_value = format!("merge {} {}", current_value, args_content);
                } else if pipe_part.starts_with("default(") && pipe_part.ends_with(")") {
                    let args_content = &pipe_part[8..pipe_part.len() - 1]; // Remove "default(" and ")"
                    current_value = format!("default {} {}", current_value, args_content);
                }
            }

            current_value
        } else {
            content.to_string()
        }
    }

    /// Process simple expressions without pipes
    fn process_simple_expression(content: &str) -> String {
        if content.starts_with("fixtures.") {
            format!("{{{{json {}}}}}", content)
        } else if content.starts_with("params.")
            || content.starts_with("request.")
            || content.starts_with("runtime.")
        {
            format!("{{{{{}}}}}", content)
        } else if content.contains("random_string(") {
            // Handle random_string(20) -> random_string 20
            use regex::Regex;
            let random_regex = Regex::new(r"random_string\((\d+)\)").unwrap();
            let processed = random_regex.replace_all(content, "random_string $1");
            format!("{{{{{}}}}}", processed)
        } else if content == "now()" {
            format!("{{{{now}}}}")
        } else {
            format!("{{{{{}}}}}", content)
        }
    }

    /// Register built-in template helpers
    fn register_helpers(handlebars: &mut Handlebars) -> PulseResult<()> {
        // Helper for generating current timestamp
        handlebars.register_helper("now", Box::new(now_helper));

        // Helper for generating random values
        handlebars.register_helper("random", Box::new(random_helper));
        handlebars.register_helper("random_string", Box::new(random_string_helper));

        // Helper for array operations
        handlebars.register_helper("length", Box::new(length_helper));
        handlebars.register_helper("find", Box::new(find_helper));
        handlebars.register_helper("find_by_field", Box::new(find_by_field_helper));
        handlebars.register_helper("find_by_multi_field", Box::new(find_by_multi_field_helper));
        handlebars.register_helper("filter", Box::new(filter_helper));

        // Helper for string operations
        handlebars.register_helper("upper", Box::new(upper_helper));
        handlebars.register_helper("lower", Box::new(lower_helper));

        // Helper for JSON serialization
        handlebars.register_helper("json", Box::new(json_helper));

        // Helper for logical operations
        handlebars.register_helper("not", Box::new(not_helper));
        handlebars.register_helper("and", Box::new(and_helper));
        handlebars.register_helper("or", Box::new(or_helper));
        handlebars.register_helper("eq", Box::new(eq_helper));
        handlebars.register_helper("ne", Box::new(ne_helper));
        handlebars.register_helper("gt", Box::new(gt_helper));
        handlebars.register_helper("gte", Box::new(gte_helper));
        handlebars.register_helper("lt", Box::new(lt_helper));
        handlebars.register_helper("lte", Box::new(lte_helper));
        handlebars.register_helper("contains", Box::new(contains_helper));
        handlebars.register_helper("starts_with", Box::new(starts_with_helper));
        handlebars.register_helper("ends_with", Box::new(ends_with_helper));
        handlebars.register_helper("matches", Box::new(regex_match_helper));
        handlebars.register_helper("exists", Box::new(exists_helper));
        handlebars.register_helper("merge", Box::new(merge_helper));
        handlebars.register_helper("select", Box::new(select_helper));
        handlebars.register_helper("default", Box::new(default_helper));

        Ok(())
    }

    /// Render a template with the given context
    pub fn render(&self, template: &str, context: &TemplateContext) -> PulseResult<String> {
        // Pre-process template to convert pipe syntax
        let processed_template = self.preprocess_template(template);

        // Convert context to JSON for Handlebars
        let json_context = self.context_to_json(context)?;

        self.handlebars
            .render_template(&processed_template, &json_context)
            .map_err(|e| {
                PulseError::runtime_error(
                    format!("Template rendering failed: {}", e),
                    Some("Check template syntax and available context variables"),
                )
            })
    }
    /// Convert template context to JSON for Handlebars
    fn context_to_json(&self, context: &TemplateContext) -> PulseResult<Value> {
        let mut json_context = Map::new();

        // Add fixtures
        json_context.insert(
            "fixtures".to_string(),
            Value::Object(
                context
                    .fixtures
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
        );

        // Add path parameters
        json_context.insert(
            "params".to_string(),
            Value::Object(
                context
                    .params
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );

        // Add runtime data
        json_context.insert(
            "runtime".to_string(),
            Value::Object(
                context
                    .runtime
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
        );

        // Add request context
        let mut request_obj = Map::new();
        request_obj.insert(
            "method".to_string(),
            Value::String(context.request.method.clone()),
        );
        request_obj.insert(
            "path".to_string(),
            Value::String(context.request.path.clone()),
        );

        // Add query parameters
        request_obj.insert(
            "query".to_string(),
            Value::Object(
                context
                    .request
                    .query
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );

        // Add headers
        request_obj.insert(
            "headers".to_string(),
            Value::Object(
                context
                    .request
                    .headers
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );

        // Add request body if present
        if let Some(ref body) = context.request.body {
            request_obj.insert("body".to_string(), body.clone());
        }

        json_context.insert("request".to_string(), Value::Object(request_obj));

        Ok(Value::Object(json_context))
    }

    /// Compile and cache a template for better performance
    pub fn compile_template(&mut self, name: &str, template: &str) -> PulseResult<()> {
        self.handlebars
            .register_template_string(name, template)
            .map_err(|e| {
                PulseError::config_error(
                    format!("Template compilation failed for '{}': {}", name, e),
                    Some("Check template syntax"),
                )
            })
    }

    /// Render a pre-compiled template
    pub fn render_compiled(&self, name: &str, context: &TemplateContext) -> PulseResult<String> {
        let json_context = self.context_to_json(context)?;

        self.handlebars.render(name, &json_context).map_err(|e| {
            PulseError::runtime_error(
                format!("Template rendering failed for '{}': {}", name, e),
                Some("Check template syntax and available context variables"),
            )
        })
    }
}

impl TemplateContext {
    /// Create a new template context from service state and request data
    pub fn new(
        state: &ServiceState,
        path_params: &PathParameters,
        request_context: RequestContext,
    ) -> Self {
        Self {
            fixtures: state.all_fixtures().clone(),
            params: path_params.all().clone(),
            runtime: state.all_runtime_data().clone(),
            request: request_context,
        }
    }

    /// Create a minimal context for testing
    pub fn minimal() -> Self {
        Self {
            fixtures: HashMap::new(),
            params: HashMap::new(),
            runtime: HashMap::new(),
            request: RequestContext {
                method: "GET".to_string(),
                path: "/".to_string(),
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
            },
        }
    }
}

impl RequestContext {
    /// Create request context from HTTP request data
    pub fn from_request_data(
        method: String,
        path: String,
        query: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: Option<Value>,
    ) -> Self {
        Self {
            method,
            path,
            query,
            headers,
            body,
        }
    }
}

/// Built-in template helpers
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_basic_template_rendering() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::minimal();
        context.fixtures.insert("name".to_string(), json!("World"));

        let result = engine.render("Hello {{fixtures.name}}!", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[test]
    fn test_path_params_rendering() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::minimal();
        context.params.insert("id".to_string(), "123".to_string());

        let result = engine.render("User ID: {{params.id}}", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "User ID: 123");
    }

    #[test]
    fn test_request_context_rendering() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::minimal();
        context.request.method = "POST".to_string();
        context.request.path = "/users".to_string();

        let result = engine.render("{{request.method}} {{request.path}}", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "POST /users");
    }

    #[test]
    fn test_now_helper() {
        let engine = TemplateEngine::new().unwrap();
        let context = TemplateContext::minimal();

        let result = engine.render("{{now}}", &context);
        assert!(result.is_ok());
        // Just check that it produces some output (timestamp format)
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_length_helper() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::minimal();
        context
            .fixtures
            .insert("items".to_string(), json!([1, 2, 3]));

        let result = engine.render("{{length fixtures.items}}", &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "3");
    }

    #[test]
    fn test_find_helper() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::minimal();
        context.fixtures.insert(
            "users".to_string(),
            json!([
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]),
        );

        let result = engine.render("{{find fixtures.users \"id\" 1}}", &context);
        assert!(result.is_ok());
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed["name"], "Alice");
    }
}
