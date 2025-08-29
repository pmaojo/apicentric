use anyhow::Result;
use pulse::{Context, PulseResult};
use pulse::execution::TestRunner;
use std::sync::Arc;

use crate::app::run_all::RunAllTestsService;
use crate::adapters::metrics_facade::MetricsFacade;
use crate::adapters::test_runner_delegator::TestRunnerDelegator;
use crate::adapters::ui_cli::CliUiAdapter;
use crate::domain::ports::testing::{RetryPolicy, TestSpec, TestRunnerPort as _};

pub struct RunCommandHandler;

impl RunCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context, workers: usize, retries: usize) -> PulseResult<()> {
        println!("🔄 Running all tests...");
        println!("⚙️  Workers: {}, Retries: {}", workers, retries);

        // Discover test specs
        let runner = TestRunner::new(context.clone());
        let spec_paths = runner.discover_specs()?;
        println!("🧪 Found {} test files", spec_paths.len());

        if spec_paths.is_empty() {
            println!("⚠️  No test files found matching pattern");
            return Ok(());
        }

        // Integrate domain service
        let metrics = MetricsFacade::new(context.metrics_manager().clone());
        let ui = CliUiAdapter;
        let domain_runner = TestRunnerDelegator::new(context.test_runner.clone());
        let service = RunAllTestsService::new(domain_runner, metrics, ui);

        let specs: Vec<TestSpec> = spec_paths
            .into_iter()
            .map(|p| TestSpec { path: p })
            .collect();

        println!("🚀 Executing tests with {} workers...", workers);
        let retry = RetryPolicy { retries: retries as u8 };
        let results = service.run(specs, workers, retry)?;

        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        println!("\n📊 Test Results:\n   ✅ Passed: {}\n   ❌ Failed: {}\n   📈 Total: {}", passed, failed, total);
        if failed > 0 { std::process::exit(1); }
        Ok(())
    }
}
