use crate::iot::model::{DigitalTwinState, VariableValue};
use crate::iot::traits::SimulationStrategy;
use async_trait::async_trait;
use log::error;
use rhai::{Engine, Scope, AST, Map, Dynamic};
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

        // Expose full state as a map
        let mut state_map = Map::new();
        for (key, val) in &state.variables {
            let dynamic_val: Dynamic = match val {
                 VariableValue::Integer(i) => (*i).into(),
                 VariableValue::Float(f) => (*f).into(),
                 VariableValue::String(s) => s.clone().into(),
                 VariableValue::Boolean(b) => (*b).into(),
            };
            state_map.insert(key.clone().into(), dynamic_val);
        }
        scope.push("state", state_map);

        // Execute script
        // Lock engine
        let engine = self.engine.lock().unwrap();
        let result = engine.eval_ast_with_scope::<f64>(&mut scope, &self.ast);

        match result {
            Ok(new_val) => {
                state
                    .variables
                    .insert(self.variable_name.clone(), VariableValue::Float(new_val));
            }
            Err(e) => {
                error!("Script execution failed for {}: {}", self.variable_name, e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iot::model::{DigitalTwinState, VariableValue};

    #[tokio::test]
    async fn test_rhai_access_state() {
        let mut state = DigitalTwinState::default();
        state.variables.insert("heater_status".to_string(), VariableValue::Float(1.0));
        state.variables.insert("temp".to_string(), VariableValue::Float(20.0));

        // Script accesses 'heater_status' from state map
        let script = r#"
            let h = state["heater_status"];
            if h == 1.0 {
                value + 1.0
            } else {
                value
            }
        "#;

        let strategy = RhaiScriptStrategy::new(script, "temp".to_string()).unwrap();
        strategy.tick(&mut state).await.unwrap();

        let new_temp = state.variables.get("temp").unwrap().as_f64().unwrap();
        assert_eq!(new_temp, 21.0);
    }
}
