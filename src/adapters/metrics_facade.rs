use crate::domain::{entities::MetricsEvent, ports::testing::MetricsSinkPort};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct MetricsFacade {
    inner: Arc<Mutex<crate::adapters::metrics::MetricsManager>>, // existing
}

impl MetricsFacade {
    pub fn new(inner: Arc<Mutex<crate::adapters::metrics::MetricsManager>>) -> Self {
        Self { inner }
    }
}

impl MetricsSinkPort for MetricsFacade {
    fn emit(&self, event: MetricsEvent) {
        let mut guard = self.inner.lock().ok();
        if let Some(m) = guard.as_mut() {
            match event {
                MetricsEvent::TestStart { spec } => m.start_test(&spec, &spec),
                MetricsEvent::TestEnd { spec, passed, ms } => {
                    let _ = m.end_test(&spec, passed, None, Duration::from_millis(ms as u64));
                }
            }
        }
    }
}
