use tokio::sync::RwLock;

/// Service responsible for storing currently active scenario.
///
/// Extracting this logic into its own component helps the
/// `ServiceInstance` adhere to the single responsibility
/// principle by delegating scenario state management.
#[derive(Default)]
pub struct ScenarioService {
    active: RwLock<Option<String>>,
}

impl ScenarioService {
    /// Create a new scenario service with no active scenario.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the active scenario.
    pub async fn set(&self, scenario: Option<String>) {
        *self.active.write().await = scenario;
    }

    /// Retrieve the currently active scenario.
    pub async fn get(&self) -> Option<String> {
        self.active.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scenario_roundtrip() {
        let service = ScenarioService::new();
        assert_eq!(service.get().await, None);
        service.set(Some("demo".into())).await;
        assert_eq!(service.get().await.as_deref(), Some("demo"));
        service.set(None).await;
        assert_eq!(service.get().await, None);
    }
}
