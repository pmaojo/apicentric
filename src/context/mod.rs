use std::path::Path;
use std::sync::Arc;

use crate::{config, PulseError, PulseResult};

pub mod init;

/// Minimal application context containing configuration and optional API simulator.
pub struct Context {
    config: config::PulseConfig,
    api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
}

impl Context {
    pub fn config(&self) -> &config::PulseConfig {
        &self.config
    }

    pub fn api_simulator(&self) -> Option<&Arc<crate::simulator::ApiSimulatorManager>> {
        self.api_simulator.as_ref()
    }

    pub async fn start_api_simulator(&self) -> PulseResult<()> {
        if let Some(ref sim) = self.api_simulator {
            sim.start().await.map_err(|e| {
                PulseError::runtime_error(
                    format!("Failed to start API simulator: {}", e),
                    Some("Check simulator configuration and port"),
                )
            })?;
        }
        Ok(())
    }

    pub async fn stop_api_simulator(&self) -> PulseResult<()> {
        if let Some(ref sim) = self.api_simulator {
            sim.stop().await.map_err(|e| {
                PulseError::runtime_error(
                    format!("Failed to stop API simulator: {}", e),
                    None::<String>,
                )
            })?;
        }
        Ok(())
    }

    pub async fn api_simulator_status(&self) -> Option<crate::simulator::SimulatorStatus> {
        if let Some(ref sim) = self.api_simulator {
            Some(sim.get_status().await)
        } else {
            None
        }
    }

    pub fn is_api_simulator_enabled(&self) -> bool {
        if let Some(ref c) = self.config.simulator {
            c.is_enabled()
        } else {
            std::env::var("PULSE_API_SIMULATOR")
                .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
                .unwrap_or(false)
        }
    }
}

/// Builder for [`Context`].
pub struct ContextBuilder {
    config: config::PulseConfig,
    api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
}

impl ContextBuilder {
    pub fn new(config_path: &Path) -> PulseResult<Self> {
        let config = config::load_config(config_path)?;
        Ok(Self {
            config,
            api_simulator: None,
        })
    }

    pub fn with_api_simulator(
        mut self,
        api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
    ) -> Self {
        self.api_simulator = api_simulator;
        self
    }

    pub fn config(&self) -> &config::PulseConfig {
        &self.config
    }

    pub fn build(self) -> PulseResult<Context> {
        Ok(Context {
            config: self.config.clone(),
            api_simulator: self.api_simulator,
        })
    }
}

/// Execution context derived from configuration and CLI flags.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub mode: config::ExecutionMode,
    pub dry_run: bool,
    pub verbose: bool,
    pub continue_on_failure: bool,
}

impl ExecutionContext {
    pub fn new(cfg: &config::PulseConfig) -> Self {
        Self {
            mode: cfg.execution.mode.clone(),
            dry_run: cfg.execution.dry_run,
            verbose: cfg.execution.verbose,
            continue_on_failure: cfg.execution.continue_on_failure,
        }
    }

    pub fn with_mode(mut self, mode: config::ExecutionMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    pub fn is_ci_mode(&self) -> bool {
        matches!(self.mode, config::ExecutionMode::CI)
    }
    pub fn is_development_mode(&self) -> bool {
        matches!(self.mode, config::ExecutionMode::Development)
    }
    pub fn is_debug_mode(&self) -> bool {
        matches!(self.mode, config::ExecutionMode::Debug)
    }
    pub fn should_skip_server_check(&self) -> bool {
        self.is_ci_mode()
    }
    pub fn should_show_progress(&self) -> bool {
        self.is_development_mode() || self.is_debug_mode()
    }
    pub fn should_log_debug(&self) -> bool {
        self.is_debug_mode() || self.verbose
    }
}

pub use Context as AppContext;
