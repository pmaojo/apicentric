// Planned orchestration service (not yet wired). Silence dead_code until integrated.
// Integrated via commands::run (RunAllTestsService). Keep this active.

use crate::domain::{entities::*, errors::*, ports::{testing::{TestRunnerPort, MetricsSinkPort}, ui::UserInterfacePort}};

pub struct RunAllTestsService<R: TestRunnerPort, M: MetricsSinkPort, U: UserInterfacePort> {
    runner: R,
    metrics: M,
    ui: U,
}

impl<R: TestRunnerPort, M: MetricsSinkPort, U: UserInterfacePort> RunAllTestsService<R, M, U> {
    pub fn new(runner: R, metrics: M, ui: U) -> Self {
        Self {
            runner,
            metrics,
            ui,
        }
    }

    pub fn run(
        &self,
        specs: Vec<TestSpec>,
        workers: usize,
        retry: RetryPolicy,
    ) -> PulseResult<Vec<TestResult>> {
        self.ui
            .print_info(&format!("Running {} specs", specs.len()));
        let progress = self
            .ui
            .create_progress_bar(specs.len() as u64, "Running tests");
        for s in &specs {
            self.metrics.emit(MetricsEvent::TestStart {
                spec: s.path.clone(),
            });
        }
        let results = self.runner.run_specs(&specs, workers, retry)?;
        for r in &results {
            self.metrics.emit(MetricsEvent::TestEnd {
                spec: r.spec.clone(),
                passed: r.passed,
                ms: r.duration.as_millis(),
            });
            progress.inc(1);
            self.ui.print_info(&format!("Spec: {}", r.spec));
            for case in &r.test_cases {
                if case.passed {
                    self.ui
                        .print_success(&format!("  \u{2713} {}", case.name));
                } else {
                    let err = case.error.clone().unwrap_or_default();
                    self.ui
                        .print_error(&format!("  \u{2717} {} - {}", case.name, err));
                }
            }
        }
        progress.finish();
        self.ui.print_success("Test run completed");
        Ok(results)
    }
}
