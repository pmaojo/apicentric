use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use async_trait::async_trait;

/// Trait for protocol adapters (transports)
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Initialize the adapter with configuration
    async fn init(&mut self, config: &AdapterConfig) -> anyhow::Result<()>;
    /// Publish a value change to the adapter
    async fn publish(&self, key: &str, value: &VariableValue) -> anyhow::Result<()>;
    /// Subscribe to a topic/variable
    async fn subscribe(&mut self, topic: &str) -> anyhow::Result<()>;
    /// Poll for new messages
    async fn poll(&mut self) -> Option<(String, VariableValue)>;
}

/// Trait for physics simulation strategies
#[async_trait]
pub trait SimulationStrategy: Send + Sync {
    /// Advance the simulation by one tick
    async fn tick(&self, state: &mut crate::iot::model::DigitalTwinState) -> anyhow::Result<()>;
}
