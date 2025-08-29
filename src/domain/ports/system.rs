//! System-level ports for interacting with external processes and resources.

use std::time::Duration;

use crate::domain::errors::PulseResult;

/// Controls the lifecycle of external servers required for tests.
pub trait ServerControllerPort {
    fn ensure_started(&self, timeout: Duration) -> PulseResult<()>;
    fn stop(&self) -> PulseResult<()>;
}

/// Manages the lifecycle of a mock API server.
pub trait MockApiPort {
    fn start(&self) -> PulseResult<()>;
    fn stop(&self) -> PulseResult<()>;
}

/// Provides access to the current time.
pub trait Clock {
    fn now_ms(&self) -> u128;
}

/// Abstraction over file system operations used by the application.
pub trait FsPort {
    fn read_to_string(&self, path: &str) -> PulseResult<String>;
}

/// Executes external processes.
pub trait ProcessPort {
    fn run(&self, cmd: &str, args: &[&str]) -> PulseResult<(i32, String, String)>;
}

