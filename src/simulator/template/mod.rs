//! Template engine for dynamic response generation
//!
//! This module provides a comprehensive template rendering system using Handlebars
//! that supports dynamic responses based on request data, fixtures, and service state.

use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::service::state::DataBucket;
use handlebars::Handlebars;
use serde_json::{Map, Value};

pub mod context;
pub mod helpers;
pub mod preprocessor;

pub use context::{RequestContext, TemplateContext};
use helpers::{bucket::register_bucket_helpers, core::register_core_helpers};
use preprocessor::TemplatePreprocessor;

/// Template engine for rendering dynamic responses
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    preprocessor: TemplatePreprocessor,
}

/// Port trait for rendering templates
///
/// This allows higher level components to depend on an abstraction
/// rather than the concrete Handlebars implementation, following
/// the dependency inversion principle.
pub trait TemplateRenderer: Send + Sync {
    /// Render the given template with the provided context
    fn render_template(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> ApicentricResult<String>;
}

impl TemplateRenderer for TemplateEngine {
    fn render_template(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> ApicentricResult<String> {
        self.render(template, context)
    }
}

impl TemplateEngine {
    /// Create a new template engine with built-in helpers
    pub fn new() -> ApicentricResult<Self> {
        let mut handlebars = Handlebars::new();
        // Register built-in helpers
        Self::register_helpers(&mut handlebars)?;

        Ok(Self {
            handlebars,
            preprocessor: TemplatePreprocessor::default(),
        })
    }

    /// Register helpers that require access to the service data bucket
    pub fn register_bucket_helpers(&mut self, bucket: DataBucket) -> ApicentricResult<()> {
        register_bucket_helpers(&mut self.handlebars, bucket);
        Ok(())
    }

    /// Register built-in template helpers
    fn register_helpers(handlebars: &mut Handlebars) -> ApicentricResult<()> {
        helpers::faker::register(handlebars);
        helpers::math::register(handlebars);
        helpers::text::register(handlebars);
        register_core_helpers(handlebars);
        Ok(())
    }

    /// Render a template with the given context
    pub fn render(&self, template: &str, context: &TemplateContext) -> ApicentricResult<String> {
        // Pre-process template to convert pipe syntax
        let processed_template = self.preprocessor.preprocess(template);

        // Convert context to JSON for Handlebars
        let json_context = self.context_to_json(context)?;

        self.handlebars
            .render_template(&processed_template, &json_context)
            .map_err(|e| {
                ApicentricError::runtime_error(
                    format!("Template rendering failed: {}", e),
                    Some("Check template syntax and available context variables"),
                )
            })
    }

    /// Convert template context to JSON for Handlebars
    fn context_to_json(&self, context: &TemplateContext) -> ApicentricResult<Value> {
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

        // Add environment variables
        json_context.insert(
            "env".to_string(),
            Value::Object(
                context
                    .env
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
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
    pub fn compile_template(&mut self, name: &str, template: &str) -> ApicentricResult<()> {
        self.handlebars
            .register_template_string(name, template)
            .map_err(|e| {
                ApicentricError::config_error(
                    format!("Template compilation failed for '{}': {}", name, e),
                    Some("Check template syntax"),
                )
            })
    }

    /// Render a pre-compiled template
    pub fn render_compiled(
        &self,
        name: &str,
        context: &TemplateContext,
    ) -> ApicentricResult<String> {
        let json_context = self.context_to_json(context)?;

        self.handlebars.render(name, &json_context).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Template rendering failed for '{}': {}", name, e),
                Some("Check template syntax and available context variables"),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::service::state::DataBucket;
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

    #[test]
    fn test_faker_helper() {
        let engine = TemplateEngine::new().unwrap();
        let context = TemplateContext::minimal();
        let result = engine
            .render("{{faker \"internet.email\"}}", &context)
            .unwrap();
        assert!(result.contains("@"));
    }

    #[test]
    fn test_bucket_helpers() {
        let mut engine = TemplateEngine::new().unwrap();
        let bucket = DataBucket::new(None);
        engine.register_bucket_helpers(bucket.clone()).unwrap();
        let context = TemplateContext::minimal();
        let result = engine
            .render("{{bucket.set \"foo\" 42}}{{bucket.get \"foo\"}}", &context)
            .unwrap();
        assert_eq!(result, "42");
        assert_eq!(bucket.get("foo"), Some(json!(42)));
    }
}
