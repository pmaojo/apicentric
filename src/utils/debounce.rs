use std::time::{Duration, Instant};

pub struct Debouncer {
    last_call: Option<Instant>,
    delay: Duration,
}

impl Debouncer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            last_call: None,
            delay: Duration::from_millis(delay_ms),
        }
    }

    pub fn should_run(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last) = self.last_call {
            if now.duration_since(last) < self.delay {
                return false;
            }
        }

        self.last_call = Some(now);
        true
    }

    pub fn reset(&mut self) {
        self.last_call = None;
    }
}
