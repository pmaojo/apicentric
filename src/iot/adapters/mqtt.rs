use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use crate::iot::traits::ProtocolAdapter;
use async_trait::async_trait;
use log::info;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;

/// An adapter that publishes digital twin state changes to an MQTT broker.
pub struct MqttAdapter {
    client: Option<AsyncClient>,
    topic_prefix: String,
}

impl MqttAdapter {
    /// Create a new MQTT Adapter instance
    pub fn new() -> Self {
        Self {
            client: None,
            topic_prefix: "".to_string(),
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

        // Spawn event loop handler
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(_) => {}
                    Err(_e) => {
                        // Connection errors are expected during reconnections
                        // Just log trace to avoid noise
                        // error!("MQTT connection error: {:?}", e);
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
}
