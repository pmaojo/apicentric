use crate::domain::errors::PulseResult;
use crate::domain::ports::testing::TestRunnerPort;
use crate::domain::entities::{RetryPolicy, TestResult, TestSpec};
use std::sync::Arc;

pub struct TestRunnerDelegator {
    inner: Arc<dyn TestRunnerPort + Send + Sync>,
}

impl TestRunnerDelegator {
    pub fn new(inner: Arc<dyn TestRunnerPort + Send + Sync>) -> Self { Self { inner } }
}

impl TestRunnerPort for TestRunnerDelegator {
    fn run_specs(
        &self,
        specs: &[TestSpec],
        workers: usize,
        retry: RetryPolicy,
    ) -> PulseResult<Vec<TestResult>> {
        self.inner.run_specs(specs, workers, retry)
    }
}
