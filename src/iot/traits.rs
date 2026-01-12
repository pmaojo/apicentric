use crate::iot::config::AdapterConfig;
use crate::iot::model::VariableValue;
use async_trait::async_trait;

#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    async fn init(&mut self, config: &AdapterConfig) -> anyhow::Result<()>;
    async fn publish(&self, key: &str, value: &VariableValue) -> anyhow::Result<()>;
    // async fn on_command(&self) -> Option<Command>; // Simplified for now
}

#[async_trait]
pub trait SimulationStrategy: Send + Sync {
    async fn tick(&self, state: &mut crate::iot::model::DigitalTwinState) -> anyhow::Result<()>;
}
