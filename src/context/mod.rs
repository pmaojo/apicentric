#[cfg(feature = "simulator")]
use std::sync::Arc;

use crate::config;
use crate::ApicentricResult;
#[cfg(feature = "simulator")]
use crate::ApicentricError;

pub mod init;

/// Minimal application context containing configuration and optional API simulator.
#[derive(Clone)]
pub struct Context {
    config: config::ApicentricConfig,
    #[cfg(feature = "simulator")]
    api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
}

impl Context {
    pub fn config(&self) -> &config::ApicentricConfig {
        &self.config
    }

    #[cfg(feature = "simulator")]
    pub fn api_simulator(&self) -> Option<&Arc<crate::simulator::ApiSimulatorManager>> {
        self.api_simulator.as_ref()
    }

    #[cfg(feature = "simulator")]
    pub async fn start_api_simulator(&self) -> ApicentricResult<()> {
        if let Some(ref sim) = self.api_simulator {
            sim.start().await.map_err(|e| {
                ApicentricError::runtime_error(
                    format!("Failed to start API simulator: {}", e),
                    Some("Check simulator configuration and port"),
                )
            })?;
        }
        Ok(())
    }

    #[cfg(feature = "simulator")]
    pub async fn stop_api_simulator(&self) -> ApicentricResult<()> {
        if let Some(ref sim) = self.api_simulator {
            sim.stop().await.map_err(|e| {
                ApicentricError::runtime_error(
                    format!("Failed to stop API simulator: {}", e),
                    None::<String>,
                )
            })?;
        }
        Ok(())
    }

    #[cfg(feature = "simulator")]
    pub async fn api_simulator_status(&self) -> Option<crate::simulator::SimulatorStatus> {
        if let Some(ref sim) = self.api_simulator {
            Some(sim.get_status().await)
        } else {
            None
        }
    }

    pub fn is_api_simulator_enabled(&self) -> bool {
        #[cfg(feature = "simulator")]
        if let Some(ref c) = self.config.simulator {
            return c.is_enabled();
        }
        false
    }
}

/// Builder for [`Context`].
pub struct ContextBuilder {
    config: config::ApicentricConfig,
    #[cfg(feature = "simulator")]
    api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
}

impl ContextBuilder {
    pub fn new(config: config::ApicentricConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "simulator")]
            api_simulator: None,
        }
    }

    #[cfg(feature = "simulator")]
    pub fn with_api_simulator(
        mut self,
        api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
    ) -> Self {
        self.api_simulator = api_simulator;
        self
    }

    pub fn config(&self) -> &config::ApicentricConfig {
        &self.config
    }

    pub fn build(self) -> ApicentricResult<Context> {
        Ok(Context {
            config: self.config.clone(),
            #[cfg(feature = "simulator")]
            api_simulator: self.api_simulator,
        })
    }
}

/// Simple execution context for CLI operations.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub mode: config::ExecutionMode,
    pub dry_run: bool,
    pub verbose: bool,
    pub continue_on_failure: bool,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            mode: config::ExecutionMode::Development,
            dry_run: false,
            verbose: false,
            continue_on_failure: false,
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

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

pub use Context as AppContext;
