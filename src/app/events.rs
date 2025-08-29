// This module defines app-level events that are planned for future TUI flows.
// Suppress dead_code warnings until they are wired into handlers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    TestStarted {
        spec: String,
    },
    TestFinished {
        spec: String,
        passed: bool,
        ms: u128,
    },
}
