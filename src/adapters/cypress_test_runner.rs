use crate::domain::{entities::*, errors::*, ports::testing::TestRunnerPort};
use crate::TestRunnerPort as LegacyTestRunnerPort;
use std::sync::Arc;
use std::time::Duration;

// Thin wrapper around existing CypressAdapter implementing the new domain port
pub struct CypressTestRunner {
    inner: Arc<crate::adapters::cypress::CypressAdapter>,
}

impl CypressTestRunner {
    pub fn new(inner: Arc<crate::adapters::cypress::CypressAdapter>) -> Self {
        Self { inner }
    }
}

impl TestRunnerPort for CypressTestRunner {
    fn run_specs(
        &self,
        specs: &[TestSpec],
        workers: usize,
        retry: RetryPolicy,
    ) -> PulseResult<Vec<TestResult>> {
        let spec_paths: Vec<String> = specs.iter().map(|s| s.path.clone()).collect();
        let results = LegacyTestRunnerPort::run_specs(
            &*self.inner,
            &spec_paths,
            workers,
            retry.retries,
            true,
        )
        .map_err(|e| PulseError::Runtime(e.to_string()))?;
        Ok(results
            .into_iter()
            .map(|(spec, passed, ms, err, cases)| TestResult {
                spec,
                passed,
                duration: Duration::from_millis(ms as u64),
                error: err,
                test_cases: cases,
            })
            .collect())
    }
}
