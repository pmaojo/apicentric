use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use crate::iot::traits::ProtocolAdapter;
use async_trait::async_trait;
use log::{error, info};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

type MqttRx = Arc<Mutex<mpsc::Receiver<(String, VariableValue)>>>;

/// An adapter that publishes digital twin state changes to an MQTT broker.
pub struct MqttAdapter {
    client: Option<AsyncClient>,
    topic_prefix: String,
    rx: Option<MqttRx>,
}

impl Default for MqttAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MqttAdapter {
    /// Create a new MQTT Adapter instance
    pub fn new() -> Self {
        Self {
            client: None,
            topic_prefix: "".to_string(),
            rx: None,
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
        self.client = Some(client);

        let (tx, rx) = mpsc::channel(100);
        self.rx = Some(Arc::new(Mutex::new(rx)));
        let prefix = self.topic_prefix.clone();

        // Spawn event loop handler
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        if let Event::Incoming(Packet::Publish(p)) = event {
                            let topic = p.topic;
                            // Remove prefix if present to get the variable name
                            // e.g. sensors/temp -> temp
                            let key = if topic.starts_with(&prefix) {
                                topic[prefix.len()..].trim_start_matches('/').to_string()
                            } else {
                                topic
                            };

                            let payload = String::from_utf8_lossy(&p.payload).to_string();
                            let value = if let Ok(i) = payload.parse::<i64>() {
                                VariableValue::Integer(i)
                            } else if let Ok(f) = payload.parse::<f64>() {
                                VariableValue::Float(f)
                            } else if let Ok(b) = payload.parse::<bool>() {
                                VariableValue::Boolean(b)
                            } else {
                                VariableValue::String(payload)
                            };

                            if let Err(e) = tx.send((key, value)).await {
                                error!("Failed to send MQTT message to channel: {}", e);
                            }
                        }
                    }
                    Err(_e) => {
                        // Connection errors are expected during reconnections
                        // Just log trace to avoid noise
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        info!(
            "MQTT Adapter initialized, connected to {}:{}",
            broker_url, port
        );
        Ok(())
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

    async fn subscribe(&mut self, topic: &str) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            let full_topic = format!("{}/{}", self.topic_prefix, topic);
            client.subscribe(full_topic, QoS::AtLeastOnce).await?;
        }
        Ok(())
    }

    async fn poll(&mut self) -> Option<(String, VariableValue)> {
        if let Some(rx_mutex) = &self.rx {
            let mut rx = rx_mutex.lock().await;
            rx.try_recv().ok()
        } else {
            None
        }
    }
}
