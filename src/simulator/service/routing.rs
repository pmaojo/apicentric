use crate::simulator::config::EndpointDefinition;
use std::collections::HashMap;

/// Extracted path parameters from a request
#[derive(Debug, Clone)]
pub struct PathParameters {
    params: HashMap<String, String>,
}

impl PathParameters {
    pub fn new() -> Self {
        Self { params: HashMap::new() }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.params.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn all(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

/// Route matching result with extracted parameters
#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub endpoint: EndpointDefinition,
    pub endpoint_index: usize,
    pub path_params: PathParameters,
}
