use crate::domain::errors::PulseResult;
use crate::domain::ports::testing::{ChangeDetectorPort, RouteIndexerPort, TestRunnerPort, WatcherPort};
use crate::domain::entities::{RetryPolicy, TestSpec};
use std::sync::Arc;

pub struct WatchAndRunImpactedService {
    watcher: Arc<dyn WatcherPort + Send + Sync>,
    change_detector: Arc<dyn ChangeDetectorPort + Send + Sync>,
    indexer: Arc<dyn RouteIndexerPort + Send + Sync>,
    runner: Arc<dyn TestRunnerPort + Send + Sync>,
}

impl WatchAndRunImpactedService {
    pub fn new(
        watcher: Arc<dyn WatcherPort + Send + Sync>,
        change_detector: Arc<dyn ChangeDetectorPort + Send + Sync>,
        indexer: Arc<dyn RouteIndexerPort + Send + Sync>,
        runner: Arc<dyn TestRunnerPort + Send + Sync>,
    ) -> Self {
        Self { watcher, change_detector, indexer, runner }
    }

    /// Watches for file changes and runs impacted tests using domain ports.
    pub async fn run(&self, root: &str, workers: usize, retry: RetryPolicy) -> PulseResult<()> {
        println!("👀 Watch (domain service) on: {}", root);
        let mut rx = self.watcher.watch(root).await?;
        while let Some(_evt) = rx.recv().await {
            let changed = self.change_detector.changed_files()?;
            if changed.is_empty() {
                println!("ℹ️  No changes detected");
                continue;
            }
            let impacted = self.indexer.map_changes_to_specs(&changed)?;
            if impacted.is_empty() {
                println!("ℹ️  No impacted specs");
                continue;
            }
            println!("🎯 Impacted: {}", impacted.len());
            let specs: Vec<TestSpec> = impacted.into_iter().map(|p| TestSpec { path: p }).collect();
            let results = self.runner.run_specs(&specs, workers, retry)?;
            let total = results.len();
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = total - passed;
            println!("📊 Results: ✅ {} / ❌ {} ({} total)", passed, failed, total);
        }
        Ok(())
    }
}
