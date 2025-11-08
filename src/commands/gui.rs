use crate::errors::{ApicentricError, ApicentricResult};
use std::process::Command;

/// Launch the Apicentric GUI application built with Tauri
pub fn gui_command() -> ApicentricResult<()> {
    let status = Command::new("cargo")
        .args(["run", "--manifest-path", "gui/Cargo.toml"])
        .status()
        .map_err(|e| ApicentricError::runtime_error(format!("Failed to launch GUI: {}", e), None::<String>))?;

    if status.success() {
        Ok(())
    } else {
        Err(ApicentricError::runtime_error(
            format!("GUI exited with status {}", status),
            None::<String>,
        ))
    }
}
