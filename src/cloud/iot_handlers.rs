use crate::iot::config::TwinConfig;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "twin"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Handler to get the IoT System Graph
/// Scans the configured library directory for Twin YAMLs
pub async fn get_iot_graph() -> Json<GraphResponse> {
    // Ideally this path should be configurable via State, but for now we default to expected locations
    // We check both "examples/iot" (dev) and "./assets/library" (prod default)
    let paths = vec!["examples/iot", "./assets/library"];

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for path_str in paths {
        let path = PathBuf::from(path_str);
        if !path.exists() {
            continue;
        }

        if let Ok(mut entries) = tokio::fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                     if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        // Try to parse as TwinConfig
                        if let Ok(config) = serde_yaml::from_str::<TwinConfig>(&content) {
                            let twin_name = config.twin.name.clone();

                            // Add Node
                            nodes.push(GraphNode {
                                id: twin_name.clone(),
                                label: twin_name.clone(),
                                node_type: "twin".to_string(),
                            });

                            // Parse Edges from Subscriptions
                            for transport in config.twin.transports {
                                if let Some(subs) = transport.params.get("subscriptions").and_then(|v| v.as_sequence()) {
                                    for sub in subs {
                                        let topic = if let Some(s) = sub.as_str() {
                                            s.to_string()
                                        } else if let Some(map) = sub.as_mapping() {
                                            map.get(&serde_yaml::Value::String("topic".to_string()))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string()
                                        } else {
                                            continue;
                                        };

                                        // Try to deduce source twin from topic convention "devices/{source_name}/..."
                                        // If topic is "devices/heater/status", source is likely "Heater" (or similar)
                                        // For MVP, if we can match the topic prefix to another node name, we link it.
                                        // Or we just create a "Topic" node if we can't find a twin?
                                        // Let's try to parse "devices/{name}/..."

                                        let parts: Vec<&str> = topic.split('/').collect();
                                        if parts.len() >= 2 && parts[0] == "devices" {
                                            let source_twin_guess = parts[1]; // e.g. "heater"

                                            // Edge: Source(Heater) -> Target(Thermostat)
                                            // Make case insensitive match later? For now assume strict or simple convention.
                                            // Capitalize first letter to match "Heater" if "heater" is in topic?
                                            // Let's just use the topic component as the source ID for now.

                                            // Check if we need to capitalize to match our Node IDs (which are "Heater", "Thermostat")
                                            // Quick hack: capitalize first letter
                                            let mut source_id = source_twin_guess.to_string();
                                            if let Some(r) = source_id.get_mut(0..1) {
                                                r.make_ascii_uppercase();
                                            }

                                            edges.push(GraphEdge {
                                                id: format!("{}-{}", source_id, twin_name),
                                                source: source_id,
                                                target: twin_name.clone(),
                                                label: Some(topic.clone()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                     }
                }
            }
        }
    }

    Json(GraphResponse { nodes, edges })
}
