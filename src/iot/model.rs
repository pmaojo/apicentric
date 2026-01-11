use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl VariableValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            VariableValue::Integer(i) => Some(*i as f64),
            VariableValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalTwinState {
    pub variables: HashMap<String, VariableValue>,
}

impl Default for DigitalTwinState {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

pub struct DigitalTwin {
    pub id: String,
    pub state: DigitalTwinState,
}

impl DigitalTwin {
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: DigitalTwinState::default(),
        }
    }
}
