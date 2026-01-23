use rhai::{Engine, Scope, Dynamic, ImmutableString};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use crate::errors::{ApicentricError, ApicentricResult};

/// Wrapper around Rhai engine to provide consistent scripting environment
pub struct ScriptingEngine {
    engine: Arc<Mutex<Engine>>,
}

impl ScriptingEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register standard functions equivalent to common JS needs

        // Console object helpers
        // Note: Rhai standard lib has 'print', so we overwrite/augment it or provide aliases
        engine.register_fn("console_log", |s: &str| {
            println!("[SCRIPT] {}", s);
        });

        // Alias for JS compatibility 'log'
        engine.register_fn("log", |s: &str| {
            println!("[SCRIPT] {}", s);
        });

        // Date/Time helpers
        // Use ImmutableString for better compatibility with Rhai
        engine.register_fn("now", || -> ImmutableString {
            chrono::Utc::now().to_rfc3339().into()
        });

        Self {
            engine: Arc::new(Mutex::new(engine)),
        }
    }

    /// Execute a script with the given context
    pub fn execute(
        &self,
        script: &str,
        context: Value
    ) -> ApicentricResult<Value> {
        let engine = self.engine.lock().unwrap();
        let mut scope = Scope::new();

        // Convert context to Rhai Dynamic
        let ctx_dynamic = rhai::serde::to_dynamic(&context).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to convert context to script object: {}", e),
                None::<String>,
            )
        })?;

        // Expose `ctx` in the scope
        scope.push("ctx", ctx_dynamic);

        // Execute the script
        let result: Dynamic = engine.eval_with_scope(&mut scope, script).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Script execution error: {}", e),
                Some("Check script syntax (Rhai)".to_string()),
            )
        })?;

        // Convert result back to JSON
        let result_json: Value = rhai::serde::from_dynamic(&result).map_err(|e| {
            ApicentricError::runtime_error(
                format!("Failed to convert script result to JSON: {}", e),
                Some("Ensure script returns a valid object/value".to_string()),
            )
        })?;

        Ok(result_json)
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
            #{
                result: "success",
                value: 42
            }
        "#;

        let context = json!({});
        let result = engine.execute(script, context).unwrap();

        assert_eq!(result["result"], "success");
        assert_eq!(result["value"], 42);
    }

    #[test]
    fn test_rhai_scripting_context_access() {
        let engine = ScriptingEngine::new();

        let script = r#"
            let user_id = ctx.request.body.id;
            let db_name = ctx.fixtures.config.name;

            #{
                processed_id: user_id + 100,
                source: db_name
            }
        "#;

        let context = json!({
            "request": {
                "body": { "id": 123 }
            },
            "fixtures": {
                "config": { "name": "test_db" }
            }
        });

        let result = engine.execute(script, context).unwrap();

        assert_eq!(result["processed_id"], 223);
        assert_eq!(result["source"], "test_db");
    }

    #[test]
    fn test_rhai_helpers() {
        let engine = ScriptingEngine::new();

        // Simplified test script without print to verify now()
        let script = r#"
            let t = now();
            console_log("Time is: " + t);

            #{
                has_time: t.len() > 0
            }
        "#;

        let context = json!({});
        let result = engine.execute(script, context).unwrap();

        assert_eq!(result["has_time"], true);
    }
}
