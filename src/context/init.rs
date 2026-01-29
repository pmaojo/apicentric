#[cfg(feature = "simulator")]
use std::sync::Arc;

#[cfg(feature = "simulator")]
use crate::config::ApicentricConfig;

/// Build the API simulator manager if enabled in configuration.
#[cfg(feature = "simulator")]
pub fn build_api_simulator(
    cfg: &ApicentricConfig,
) -> Option<Arc<crate::simulator::ApiSimulatorManager>> {
    if let Some(ref simulator_config) = cfg.simulator {
        if simulator_config.is_enabled() {
            return Some(Arc::new(crate::simulator::ApiSimulatorManager::new(
                simulator_config.clone(),
            )));
        }
    }
    None
}
