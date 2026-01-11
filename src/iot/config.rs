use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinConfig {
    pub twin: TwinDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinDefinition {
    pub name: String,
    pub physics: Vec<PhysicsConfig>,
    pub transports: Vec<AdapterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub variable: String,
    pub strategy: String,
    pub params: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    #[serde(rename = "type")]
    pub adapter_type: String,
    #[serde(flatten)]
    pub params: HashMap<String, serde_yaml::Value>,
}
