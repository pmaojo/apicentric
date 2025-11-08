use crate::errors::{ApicentricError, ApicentricResult};
use std::process::Command;

/// Launch the Apicentric GUI application built with Tauri
pub fn gui_command() -> ApicentricResult<()> {
    let status = Command::new("cargo")
        .args(["run", "--manifest-path", "gui/Cargo.toml"])
        .status()
        .map_err(|e| ApicentricError::runtime_error(
            format!("Failed to launch GUI: {}", e),
            Some("Ensure the GUI component is installed and Cargo is available")
        ))?;

    if status.success() {
        Ok(())
    } else {
        Err(ApicentricError::runtime_error(
            format!("GUI exited with status {}", status),
            Some("Check the GUI logs for errors or try running 'cargo run --manifest-path gui/Cargo.toml' manually")
        ))
    }
}
