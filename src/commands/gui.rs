use crate::errors::{PulseError, PulseResult};
use std::process::Command;

/// Launch the Pulse GUI application built with Tauri
pub fn gui_command() -> PulseResult<()> {
    let status = Command::new("cargo")
        .args(["run", "--manifest-path", "gui/Cargo.toml"])
        .status()
        .map_err(|e| PulseError::runtime_error(format!("Failed to launch GUI: {}", e), None::<String>))?;

    if status.success() {
        Ok(())
    } else {
        Err(PulseError::runtime_error(
            format!("GUI exited with status {}", status),
            None::<String>,
        ))
    }
}
