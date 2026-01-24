use crate::errors::ApicentricResult;
use rhai::{Engine, Scope, AST};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A thread-safe scripting engine for simulation logic
pub struct ScriptingEngine {
    engine: Arc<Mutex<Engine>>,
    cache: Arc<Mutex<HashMap<String, AST>>>,
}

impl Default for ScriptingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptingEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register standard functions
        engine.register_fn("log", |s: &str| println!("Script log: {}", s));
        engine.register_fn("console_log", |s: &str| println!("Script console: {}", s));
        engine.register_fn("print", |s: &str| println!("Script print: {}", s));
        engine.register_fn("now", || chrono::Utc::now().to_rfc3339());

        Self {
            engine: Arc::new(Mutex::new(engine)),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Execute a script in the context of a request
    pub fn execute(&self, script: &str, context: &Value) -> ApicentricResult<Value> {
        let engine = self.engine.lock().unwrap();
        let mut cache = self.cache.lock().unwrap();

        // Compile or retrieve from cache
        let ast = if let Some(ast) = cache.get(script) {
            ast.clone()
        } else {
            let ast = engine.compile(script)?;
            cache.insert(script.to_string(), ast.clone());
            ast
        };

        // Create scope with context
        let mut scope = Scope::new();
        let dynamic_ctx = rhai::serde::to_dynamic(context)?;
        scope.push("ctx", dynamic_ctx);

        // Execute
        // Explicitly specify Dynamic as the return type
        let result = engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast)?;

        // Convert result back to JSON value
        let json_val: Value = rhai::serde::from_dynamic(&result)?;
        Ok(json_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rhai_scripting_basic() {
        let engine = ScriptingEngine::new();
        let script = r#"
            let x = 10;
            let y = 20;
            x + y
        "#;
        let context = json!({});
        let result = engine.execute(script, &context).unwrap();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_rhai_scripting_context_access() {
        let engine = ScriptingEngine::new();
        let script = r#"
            ctx.request.method
        "#;
        let context = json!({
            "request": {
                "method": "POST"
            }
        });
        let result = engine.execute(script, &context).unwrap();
        assert_eq!(result, "POST");
    }

    #[test]
    fn test_rhai_helpers() {
        let engine = ScriptingEngine::new();
        // Simplified script to rule out print/block weirdness
        let script = r#"
            let t = now();
            t
        "#;
        let context = json!({});
        let result = engine.execute(script, &context).unwrap();
        assert!(result.is_string());
    }
}
