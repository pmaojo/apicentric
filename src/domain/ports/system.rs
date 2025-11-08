//! System-level ports for interacting with external processes and resources.

use std::time::Duration;

use crate::domain::errors::ApicentricResult;

/// Controls the lifecycle of external servers required for tests.
pub trait ServerControllerPort {
    fn ensure_started(&self, timeout: Duration) -> ApicentricResult<()>;
    fn stop(&self) -> ApicentricResult<()>;
}

/// Manages the lifecycle of the simulator.
pub trait MockApiPort {
    fn start(&self) -> ApicentricResult<()>;
    fn stop(&self) -> ApicentricResult<()>;
}

/// Provides access to the current time.
pub trait Clock {
    fn now_ms(&self) -> u128;
}

/// Abstraction over file system operations used by the application.
pub trait FsPort {
    fn read_to_string(&self, path: &str) -> ApicentricResult<String>;
}

/// Executes external processes.
pub trait ProcessPort {
    fn run(&self, cmd: &str, args: &[&str]) -> ApicentricResult<(i32, String, String)>;
}

