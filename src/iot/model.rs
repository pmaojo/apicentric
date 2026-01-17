use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a value for a device variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl VariableValue {
    /// Try to convert the value to f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            VariableValue::Integer(i) => Some(*i as f64),
            VariableValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

/// The state of a digital twin, containing all variables
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DigitalTwinState {
    pub variables: HashMap<String, VariableValue>,
}

/// A Digital Twin instance
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
