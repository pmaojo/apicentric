use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use crate::iot::traits::ProtocolAdapter;
use async_trait::async_trait;
use log::{info, trace};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;

/// An adapter that publishes digital twin state changes to an MQTT broker.
pub struct MqttAdapter {
    client: Option<AsyncClient>,
    topic_prefix: String,
    rx: Option<mpsc::Receiver<(String, VariableValue)>>,
    subscription_map: HashMap<String, String>,
}

impl MqttAdapter {
    /// Create a new MQTT Adapter instance
    pub fn new() -> Self {
        Self {
            client: None,
            topic_prefix: "".to_string(),
            rx: None,
            subscription_map: HashMap::new(),
        }
    }
}

#[async_trait]
impl ProtocolAdapter for MqttAdapter {
    async fn init(&mut self, config: &AdapterConfig) -> anyhow::Result<()> {
        let broker_url = config
            .params
            .get("broker_url")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost");
        let port = config
            .params
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(1883) as u16;
        let client_id = config
            .params
            .get("client_id")
            .and_then(|v| v.as_str())
            .unwrap_or("apicentric-twin");
        self.topic_prefix = config
            .params
            .get("topic_prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("sensors")
            .to_string();

        let mut mqttoptions = MqttOptions::new(client_id, broker_url, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        self.client = Some(client.clone());

        let (tx, rx) = mpsc::channel(100);
        self.rx = Some(rx);

        // Spawn event loop handler
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::Publish(p))) => {
                         if let Ok(payload_str) = std::str::from_utf8(&p.payload) {
                            let value = if let Ok(f) = payload_str.parse::<f64>() {
                                VariableValue::Float(f)
                            } else if let Ok(b) = payload_str.parse::<bool>() {
                                VariableValue::Boolean(b)
                            } else {
                                VariableValue::String(payload_str.to_string())
                            };

                            let _ = tx.send((p.topic, value)).await;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        // Connection errors are expected during reconnections
                        trace!("MQTT connection loop error: {:?}", e);
                        // Optional: Add delay to prevent tight loop on hard fail
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        info!(
            "MQTT Adapter initialized, connected to {}:{}",
            broker_url, port
        );

        // Handle Subscriptions
        if let Some(subs) = config.params.get("subscriptions").and_then(|v| v.as_sequence()) {
            for sub in subs {
                // Support both string "topic" and object { topic: "...", variable: "..." }
                let (topic, var_name) = if let Some(s) = sub.as_str() {
                    (s.to_string(), s.to_string())
                } else if let Some(map) = sub.as_mapping() {
                    let t = map.get(&serde_yaml::Value::String("topic".to_string()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let v = map.get(&serde_yaml::Value::String("variable".to_string()))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&t)
                        .to_string();
                    (t, v)
                } else {
                    continue;
                };

                if !topic.is_empty() {
                    self.subscription_map.insert(topic.clone(), var_name);
                    client.subscribe(topic, QoS::AtLeastOnce).await?;
                }
            }
        }

        Ok(())
    }

    async fn subscribe(&mut self, topics: &[String]) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            for topic in topics {
                // For manual subscribe calls, we assume var name = topic
                self.subscription_map.insert(topic.clone(), topic.clone());
                client.subscribe(topic, QoS::AtLeastOnce).await?;
            }
        }
        Ok(())
    }

    async fn poll(&mut self) -> anyhow::Result<Vec<(String, VariableValue)>> {
        let mut results = Vec::new();
        if let Some(rx) = &mut self.rx {
            while let Ok((topic, value)) = rx.try_recv() {
                // Map topic to variable name
                let var_name = self.subscription_map.get(&topic).unwrap_or(&topic).clone();
                results.push((var_name, value));
            }
        }
        Ok(results)
    }

    async fn publish(&self, key: &str, value: &VariableValue) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            let topic = format!("{}/{}", self.topic_prefix, key);
            let payload = match value {
                VariableValue::Integer(v) => v.to_string(),
                VariableValue::Float(v) => v.to_string(),
                VariableValue::String(v) => v.clone(),
                VariableValue::Boolean(v) => v.to_string(),
            };

            client
                .publish(topic, QoS::AtLeastOnce, false, payload)
                .await?;
        }
        Ok(())
    }
}
