pub mod init;

use std::path::Path;
use std::sync::Arc;

use crate::config;
use crate::domain::ports::testing::{ChangeDetectorPort, RouteIndexerPort, WatcherPort};
use crate::execution::ExecutionContext;
use crate::{PulseError, PulseResult, TestRunnerPort};

/// Contenedor IoC sencillo para dependencias basadas en puertos.
pub struct Context {
    config: config::PulseConfig,
    execution_context: ExecutionContext,
    pub change_detector: Arc<dyn ChangeDetectorPort + Send + Sync>,
    pub route_indexer: Arc<dyn RouteIndexerPort + Send + Sync>,
    pub test_runner: Arc<dyn TestRunnerPort + Send + Sync>,
    pub watcher: Arc<dyn WatcherPort + Send + Sync>,
    pub junit_adapter: Arc<crate::adapters::junit::JUnitAdapter>,
    pub metrics_manager: Arc<std::sync::Mutex<crate::adapters::metrics::MetricsManager>>,
    pub api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
}

impl Context {
    pub fn with_execution_context(mut self, execution_context: ExecutionContext) -> Self {
        self.execution_context = execution_context;
        self
    }

    pub fn execution_context(&self) -> &ExecutionContext {
        &self.execution_context
    }
    pub fn config(&self) -> &config::PulseConfig {
        &self.config
    }
    pub fn metrics_manager(
        &self,
    ) -> &Arc<std::sync::Mutex<crate::adapters::metrics::MetricsManager>> {
        &self.metrics_manager
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

/// Builder para [`Context`].
pub struct ContextBuilder {
    config: config::PulseConfig,
    change_detector: Option<Arc<dyn ChangeDetectorPort + Send + Sync>>,
    route_indexer: Option<Arc<dyn RouteIndexerPort + Send + Sync>>,
    test_runner: Option<Arc<dyn TestRunnerPort + Send + Sync>>,
    watcher: Option<Arc<dyn WatcherPort + Send + Sync>>,
    junit_adapter: Option<Arc<crate::adapters::junit::JUnitAdapter>>,
    metrics_manager: Option<Arc<std::sync::Mutex<crate::adapters::metrics::MetricsManager>>>,
    api_simulator: Option<Arc<crate::simulator::ApiSimulatorManager>>,
}

impl ContextBuilder {
    pub fn new(config_path: &Path) -> PulseResult<Self> {
        let config = config::load_config(config_path)?;
        Ok(Self {
            config,
            change_detector: None,
            route_indexer: None,
            test_runner: None,
            junit_adapter: None,
            metrics_manager: None,
            api_simulator: None,
            watcher: None,
        })
    }

    pub fn with_change_detector(
        mut self,
        change_detector: Arc<dyn ChangeDetectorPort + Send + Sync>,
    ) -> Self {
        self.change_detector = Some(change_detector);
        self
    }

    pub fn with_route_indexer(
        mut self,
        route_indexer: Arc<dyn RouteIndexerPort + Send + Sync>,
    ) -> Self {
        self.route_indexer = Some(route_indexer);
        self
    }

    pub fn with_test_runner(mut self, test_runner: Arc<dyn TestRunnerPort + Send + Sync>) -> Self {
        self.test_runner = Some(test_runner);
        self
    }

    pub fn with_watcher(mut self, watcher: Arc<dyn WatcherPort + Send + Sync>) -> Self {
        self.watcher = Some(watcher);
        self
    }

    pub fn with_junit_adapter(
        mut self,
        junit_adapter: Arc<crate::adapters::junit::JUnitAdapter>,
    ) -> Self {
        self.junit_adapter = Some(junit_adapter);
        self
    }

    pub fn with_metrics_manager(
        mut self,
        metrics_manager: Arc<std::sync::Mutex<crate::adapters::metrics::MetricsManager>>,
    ) -> Self {
        self.metrics_manager = Some(metrics_manager);
        self
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
            execution_context: ExecutionContext::new(&self.config),
            change_detector: self.change_detector.ok_or_else(|| {
                PulseError::runtime_error("Missing change detector", None::<String>)
            })?,
            route_indexer: self.route_indexer.ok_or_else(|| {
                PulseError::runtime_error("Missing route indexer", None::<String>)
            })?,
            test_runner: self
                .test_runner
                .ok_or_else(|| PulseError::runtime_error("Missing test runner", None::<String>))?,
            junit_adapter: self.junit_adapter.ok_or_else(|| {
                PulseError::runtime_error("Missing JUnit adapter", None::<String>)
            })?,
            metrics_manager: self.metrics_manager.unwrap_or_else(|| {
                Arc::new(std::sync::Mutex::new(
                    crate::adapters::metrics::MetricsManager::new(),
                ))
            }),
            api_simulator: self.api_simulator,
            watcher: self.watcher.ok_or_else(|| {
                PulseError::runtime_error("Missing watcher", None::<String>)
            })?,
        })
    }
}

pub use Context as AppContext;
