use serde_json::Value;
use std::collections::HashMap;

use crate::simulator::service::{routing::PathParameters, state::ServiceState};

/// Template context containing all available data for rendering
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub fixtures: HashMap<String, Value>,
    pub params: HashMap<String, String>,
    pub runtime: HashMap<String, Value>,
    pub env: HashMap<String, String>,
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
            env: std::env::vars().collect(),
            request: request_context,
        }
    }

    /// Create a minimal context for testing
    pub fn minimal() -> Self {
        Self {
            fixtures: HashMap::new(),
            params: HashMap::new(),
            runtime: HashMap::new(),
            env: std::env::vars().collect(),
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
