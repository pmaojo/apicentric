use anyhow::Result;
use pulse::{Context, PulseResult};
use pulse::execution::TestRunner;
use std::time::Duration;

use crate::app::watch_impacted::WatchAndRunImpactedService;
use crate::adapters::test_runner_delegator::TestRunnerDelegator;
use crate::domain::ports::testing::{RetryPolicy, TestSpec, TestRunnerPort as _};

pub struct WatchCommandHandler;

impl WatchCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context, workers: usize, retries: usize, debounce_ms: u64) -> PulseResult<()> {
        println!("👀 Starting watch mode...");
        println!("⚙️  Workers: {}, Retries: {}, Debounce: {}ms", workers, retries, debounce_ms);
        println!("📁 Watching: {}", context.config().routes_dir);
        println!("🔍 Pattern: {}", context.config().specs_pattern);
        println!("\n💡 Press Ctrl+C to stop watching\n");

        // Initial run via existing execution runner
        self.run_initial_tests(&TestRunner::new(context.clone()), workers, retries)?;

        // Domain service watching
        let service = WatchAndRunImpactedService::new(
            context.watcher.clone(),
            context.change_detector.clone(),
            context.route_indexer.clone(),
            context.test_runner.clone(),
        );
        let retry = RetryPolicy { retries: retries as u8 };
        let root = context.config().routes_dir.clone();
        let rt = tokio::runtime::Runtime::new().map_err(|e| pulse::PulseError::Runtime(e.to_string()))?;
        rt.block_on(async move { service.run(&root, workers, retry).await })?;

        Ok(())
    }

    fn run_initial_tests(&self, runner: &TestRunner, workers: usize, retries: usize) -> PulseResult<()> {
        println!("🔍 Discovering tests...");
        
        let specs = runner.discover_specs()?;
        if specs.is_empty() {
            println!("⚠️  No test files found. Make sure your specs_pattern is correct.");
            return Ok(());
        }

        println!("🧪 Found {} test files", specs.len());
        println!("🚀 Running initial test suite...");

        let results = runner.run_tests(specs, workers, retries)?;
        let total = results.len();
        let passed = results.iter().filter(|r| r.success).count();
        let failed = total - passed;

        println!("📊 Initial results: ✅ {} passed, ❌ {} failed", passed, failed);

        Ok(())
    }
}
