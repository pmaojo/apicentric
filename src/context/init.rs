use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config::PulseConfig;
use crate::domain::ports::testing::{ChangeDetectorPort, RouteIndexerPort, WatcherPort};
use crate::execution::ExecutionContext;
use crate::TestRunnerPort;

pub fn build_metrics_manager(
    cfg: &PulseConfig,
    exec_ctx: &ExecutionContext,
) -> Arc<Mutex<crate::adapters::metrics::MetricsManager>> {
    let mut metrics_manager = crate::adapters::metrics::MetricsManager::new();

    if let Some(ref metrics_config) = cfg.metrics {
        if let Some(ref allure_config) = metrics_config.allure {
            if allure_config.enabled {
                if let Ok(allure) = crate::adapters::metrics::AllureAdapter::new(PathBuf::from(
                    &allure_config.report_dir,
                )) {
                    metrics_manager = metrics_manager.with_allure(allure);
                }
            }
        }

        if let Some(ref prometheus_config) = metrics_config.prometheus {
            if prometheus_config.enabled && !exec_ctx.dry_run {
                if let Ok(prom) =
                    crate::adapters::metrics::PrometheusAdapter::new(prometheus_config.port)
                {
                    metrics_manager = metrics_manager.with_prometheus(prom);
                }
            }
        }

        if let Some(ref sentry_config) = metrics_config.sentry {
            if sentry_config.enabled {
                if let Ok(sentry) = crate::adapters::metrics::SentryAdapter::new(
                    &sentry_config.dsn,
                    &sentry_config.environment,
                ) {
                    metrics_manager = metrics_manager.with_sentry(sentry);
                }
            }
        }
    }

    Arc::new(Mutex::new(metrics_manager))
}

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

pub fn build_adapters(
    cfg: &PulseConfig,
) -> (
    Arc<dyn ChangeDetectorPort + Send + Sync>,
    Arc<dyn RouteIndexerPort + Send + Sync>,
    Arc<dyn TestRunnerPort + Send + Sync>,
    Arc<crate::adapters::junit::JUnitAdapter>,
    Arc<dyn WatcherPort + Send + Sync>,
) {
    let change_detector = Arc::new(crate::adapters::git::GitAdapter::new());
    let route_indexer = Arc::new(
        crate::adapters::route_indexer_adapter::RouteIndexerAdapter::new(Arc::new(
            crate::adapters::route_indexer::RouteIndexer::new(
                &cfg.routes_dir,
                &cfg.specs_dir,
                &cfg.index_cache_path,
            ),
        )),
    );
    let server_manager = Arc::new(crate::adapters::server_manager::ServerManager::new(
        cfg.server.clone(),
    ));
    let test_runner = Arc::new(
        crate::adapters::cypress::CypressAdapter::new(
            cfg.cypress_config_path.clone(),
            cfg.base_url.clone(),
        )
        .with_server_manager(
            server_manager,
            cfg.execution.mode.clone(),
            cfg.server.clone(),
        ),
    );
    let junit_adapter = Arc::new(crate::adapters::junit::JUnitAdapter::new(
        cfg.reports_dir.clone(),
    ));

    let watcher = Arc::new(crate::adapters::watcher::FileWatcher::new(500)) as Arc<dyn WatcherPort + Send + Sync>;

    (change_detector, route_indexer, test_runner, junit_adapter, watcher)
}
