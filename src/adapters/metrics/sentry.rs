use crate::PulseResult;
use sentry::{capture_message, configure_scope, ClientInitGuard, Level};
use std::collections::HashMap;
use std::time::Duration;

pub struct SentryAdapter {
    _guard: ClientInitGuard,
}

impl SentryAdapter {
    pub fn new(dsn: &str, environment: &str) -> PulseResult<Self> {
        let guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(environment.to_string().into()),
                traces_sample_rate: 0.1,
                ..Default::default()
            },
        ));

        // Set global tags
        configure_scope(|scope| {
            scope.set_tag("component", "pulse-test-runner");
            scope.set_tag("environment", environment);
        });

        println!("✅ Sentry initialized for environment: {}", environment);

        Ok(Self { _guard: guard })
    }

    pub fn report_test_failure(&self, spec: &str, error: &str, duration: Duration) {
        configure_scope(|scope| {
            scope.set_tag("test_spec", spec);
            scope.set_extra("duration_ms", (duration.as_millis() as u64).into());
            scope.set_extra("test_type", "cypress".into());
            scope.set_level(Some(Level::Error));
        });

        let event_id = capture_message(
            &format!("Test failure in {}: {}", spec, error),
            Level::Error,
        );

        // Also send structured event
        sentry::with_scope(
            |scope| {
                scope.set_tag("event_type", "test_failure");
                scope.set_extra("spec_file", spec.into());
                scope.set_extra("error_message", error.into());
                scope.set_extra("duration_seconds", duration.as_secs_f64().into());
            },
            || {
                sentry::capture_event(sentry::protocol::Event {
                    message: Some(format!("Test failure in {}", spec)),
                    level: Level::Error,
                    ..Default::default()
                });
            },
        );

        println!(
            "📊 Reported test failure to Sentry: {} (Event ID: {:?})",
            spec, event_id
        );
    }

    pub fn report_flaky_test(&self, spec: &str, failure_count: usize, total_runs: usize) {
        let flaky_rate = failure_count as f64 / total_runs as f64;

        configure_scope(|scope| {
            scope.set_tag("test_spec", spec);
            scope.set_tag("event_type", "flaky_test");
            scope.set_extra("failure_count", failure_count.into());
            scope.set_extra("total_runs", total_runs.into());
            scope.set_extra("flaky_rate", flaky_rate.into());
            scope.set_level(Some(Level::Warning));
        });

        let event_id = capture_message(
            &format!(
                "Flaky test detected: {} ({}% failure rate)",
                spec,
                (flaky_rate * 100.0) as u32
            ),
            Level::Warning,
        );

        println!(
            "📊 Reported flaky test to Sentry: {} (Event ID: {:?})",
            spec, event_id
        );
    }

    pub fn report_test_suite_completion(
        &self,
        total_tests: usize,
        failed_tests: usize,
        duration: Duration,
    ) {
        let success_rate = ((total_tests - failed_tests) as f64 / total_tests as f64) * 100.0;

        configure_scope(|scope| {
            scope.set_tag("event_type", "test_suite_completion");
            scope.set_extra("total_tests", total_tests.into());
            scope.set_extra("failed_tests", failed_tests.into());
            scope.set_extra("success_rate", success_rate.into());
            scope.set_extra("duration_seconds", duration.as_secs_f64().into());
            scope.set_level(if failed_tests > 0 {
                Some(Level::Warning)
            } else {
                Some(Level::Info)
            });
        });

        let message = if failed_tests > 0 {
            format!(
                "Test suite completed with {} failures out of {} tests ({:.1}% success rate)",
                failed_tests, total_tests, success_rate
            )
        } else {
            format!(
                "Test suite completed successfully: {} tests passed",
                total_tests
            )
        };

        let event_id = capture_message(
            &message,
            if failed_tests > 0 {
                Level::Warning
            } else {
                Level::Info
            },
        );
        println!(
            "📊 Reported test suite completion to Sentry (Event ID: {:?})",
            event_id
        );
    }

    pub fn report_performance_issue(
        &self,
        issue_type: &str,
        details: &str,
        metrics: HashMap<String, f64>,
    ) {
        configure_scope(|scope| {
            scope.set_tag("event_type", "performance_issue");
            scope.set_tag("issue_type", issue_type);
            scope.set_extra("details", details.into());

            for (key, value) in &metrics {
                scope.set_extra(key, (*value).into());
            }

            scope.set_level(Some(Level::Warning));
        });

        let event_id = capture_message(
            &format!("Performance issue detected: {}", issue_type),
            Level::Warning,
        );
        println!(
            "📊 Reported performance issue to Sentry: {} (Event ID: {:?})",
            issue_type, event_id
        );
    }

    pub fn add_breadcrumb(&self, message: &str, category: &str) {
        sentry::add_breadcrumb(sentry::protocol::Breadcrumb {
            message: Some(message.to_string()),
            category: Some(category.to_string()),
            level: sentry::protocol::Level::Info,
            ..Default::default()
        });
    }

    pub fn flush(&self) -> PulseResult<()> {
        sentry::Hub::current()
            .client()
            .unwrap()
            .flush(Some(Duration::from_secs(5)));
        Ok(())
    }
}
