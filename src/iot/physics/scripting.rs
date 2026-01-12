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
