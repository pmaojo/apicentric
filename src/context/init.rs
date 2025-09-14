use std::sync::Arc;

use crate::config::PulseConfig;

/// Build the API simulator manager if enabled in configuration.
pub fn build_api_simulator(
    cfg: &PulseConfig,
) -> Option<Arc<crate::simulator::ApiSimulatorManager>> {
    if let Some(ref simulator_config) = cfg.simulator {
        let (enabled, _) = simulator_config.effective_enabled_state();
        if enabled {
            return Some(Arc::new(crate::simulator::ApiSimulatorManager::new(
                simulator_config.clone(),
            )));
        }
    } else if std::env::var("PULSE_API_SIMULATOR").is_ok() {
        let default_config = crate::simulator::config::SimulatorConfig::default_config();
        if default_config.is_enabled() {
            return Some(Arc::new(crate::simulator::ApiSimulatorManager::new(
                default_config,
            )));
        }
    }
    None
}
