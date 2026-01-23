use crate::iot::model::{DigitalTwinState, VariableValue};
use crate::iot::traits::SimulationStrategy;
use async_trait::async_trait;
use log::error;
use rhai::{Engine, Scope, AST};
use std::sync::{Arc, Mutex};

/// A simulation strategy that executes Rhai scripts to determine variable values.
pub struct RhaiScriptStrategy {
    // Engine must be thread-safe. Rhai Engine is Send+Sync with 'sync' feature,
    // but to be safe and avoid feature hell, wrapping in Arc<Mutex> works.
    // However, if we enabled 'sync', we should be good.
    // The previous error `dyn Fn cannot be sent between threads` usually relates to closures registered in Engine.
    // Since we don't register custom closures here, it might be internal.
    // Wrapping in Arc<Mutex> is the safest bet for now.
    engine: Arc<Mutex<Engine>>,
    ast: AST, // AST is Send + Sync
    variable_name: String,
}

impl RhaiScriptStrategy {
    /// Create a new script strategy from a script string
    pub fn new(script: &str, variable_name: String) -> anyhow::Result<Self> {
        let engine = Engine::new();
        let ast = engine.compile(script)?;
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            ast,
            variable_name,
        })
    }
}

#[async_trait]
impl SimulationStrategy for RhaiScriptStrategy {
    async fn tick(&self, state: &mut DigitalTwinState) -> anyhow::Result<()> {
        let mut scope = Scope::new();

        // Expose current state to script
        let current_val = state
            .variables
            .get(&self.variable_name)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        scope.push("value", current_val);

        // Expose entire state as a map
        let mut state_map = rhai::Map::new();
        for (k, v) in &state.variables {
            let val = match v {
                VariableValue::Integer(i) => rhai::Dynamic::from(*i),
                VariableValue::Float(f) => rhai::Dynamic::from(*f),
                VariableValue::String(s) => rhai::Dynamic::from(s.clone()),
                VariableValue::Boolean(b) => rhai::Dynamic::from(*b),
            };
            state_map.insert(k.clone().into(), val);
        }
        scope.push("state", state_map);

        // Execute script
        // Lock engine
        let engine = self.engine.lock().unwrap();
        let result = engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &self.ast);

        match result {
            Ok(val) => {
                let var_val = if let Some(f) = val.clone().try_cast::<f64>() {
                    VariableValue::Float(f)
                } else if let Some(i) = val.clone().try_cast::<i64>() {
                    VariableValue::Integer(i)
                } else if let Some(b) = val.clone().try_cast::<bool>() {
                    VariableValue::Boolean(b)
                } else if let Some(s) = val.clone().try_cast::<String>() {
                    VariableValue::String(s)
                } else {
                    VariableValue::String(val.to_string())
                };

                state
                    .variables
                    .insert(self.variable_name.clone(), var_val);
            }
            Err(e) => {
                error!("Script execution failed for {}: {}", self.variable_name, e);
            }
        }
        Ok(())
    }
}
