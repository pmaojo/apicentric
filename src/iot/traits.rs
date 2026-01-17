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

    /// Subscribe to specific topics/addresses
    async fn subscribe(&mut self, _topics: &[String]) -> anyhow::Result<()> {
        Ok(())
    }

    /// Poll for incoming messages/state updates from the network
    /// Returns a list of (variable_name, new_value) tuples
    async fn poll(&mut self) -> anyhow::Result<Vec<(String, VariableValue)>> {
        Ok(Vec::new())
    }
}

/// Trait for physics simulation strategies
#[async_trait]
pub trait SimulationStrategy: Send + Sync {
    /// Advance the simulation by one tick
    async fn tick(&self, state: &mut crate::iot::model::DigitalTwinState) -> anyhow::Result<()>;
}
