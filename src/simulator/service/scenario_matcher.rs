use crate::simulator::config::{
    EndpointDefinition, ResponseDefinition, ScenarioDefinition, ScenarioStrategy,
};
use crate::simulator::service::state::ServiceState;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Match a scenario based on query, header, or body conditions
pub async fn match_scenario(
    endpoint: &EndpointDefinition,
    state: &Arc<RwLock<ServiceState>>,
    endpoint_index: usize,
    active_scenario: Option<String>,
    query: &HashMap<String, String>,
    headers: &HashMap<String, String>,
    body: &Option<Value>,
) -> Option<(u16, ResponseDefinition)> {
    if let Some(scenarios) = &endpoint.scenarios {
        // First evaluate explicit conditions
        for scenario in scenarios {
            if let Some(cond) = &scenario.conditions {
                let mut matches = true;
                if let Some(q) = &cond.query {
                    for (k, v) in q {
                        if query.get(k) != Some(v) {
                            matches = false;
                            break;
                        }
                    }
                }
                if matches {
                    if let Some(h) = &cond.headers {
                        for (k, v) in h {
                            match headers.get(k) {
                                Some(val) if val.eq_ignore_ascii_case(v) => {}
                                _ => {
                                    matches = false;
                                    break;
                                }
                            }
                        }
                    }
                }
                if matches {
                    if let Some(b) = &cond.body {
                        if let Some(Value::Object(obj)) = body {
                            for (k, v) in b {
                                if obj.get(k) != Some(v) {
                                    matches = false;
                                    break;
                                }
                            }
                        } else {
                            matches = false;
                        }
                    }
                }
                if matches {
                    return Some((
                        scenario.response.status,
                        scenario.response.definition.clone(),
                    ));
                }
            }
        }
        // Fallback to manually selected scenario
        if let Some(active) = active_scenario {
            for scenario in scenarios {
                if let Some(name) = &scenario.name {
                    if *name == active {
                        return Some((
                            scenario.response.status,
                            scenario.response.definition.clone(),
                        ));
                    }
                }
            }
        }

        // Automatic rotation/random selection for scenarios without conditions or name
        let candidates: Vec<&ScenarioDefinition> = scenarios
            .iter()
            .filter(|s| s.conditions.is_none() && s.name.is_none())
            .collect();
        if !candidates.is_empty() {
            let strategy = candidates[0]
                .strategy
                .clone()
                .unwrap_or(ScenarioStrategy::Sequential);
            let index = {
                let mut guard = state.write().await;
                guard.next_response_index(endpoint_index, candidates.len(), strategy)
            };
            let scenario = candidates[index];
            return Some((
                scenario.response.status,
                scenario.response.definition.clone(),
            ));
        }
    }
    None
}
