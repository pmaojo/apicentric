use crate::errors::{ApicentricError, ApicentricResult};
use std::process::Command;
use std::thread;

/// Launch the Apicentric Web UI and its backend components
pub fn webui_command() -> ApicentricResult<()> {
    // Start the backend server (GUI application) in a separate thread
    let backend_handle = thread::spawn(|| {
        let status = Command::new("cargo")
            .args(["run", "--manifest-path", "gui/Cargo.toml"])
            .status()
            .map_err(|e| ApicentricError::runtime_error(
                format!("Failed to launch backend GUI application: {}", e),
                Some("Ensure the GUI component is installed and Cargo is available")
            ));

        if let Err(e) = status {
            eprintln!("{}", e);
        }
    });

    // Start the frontend server (Next.js) in a separate thread
    let frontend_handle = thread::spawn(|| {
        let status = Command::new("npm")
            .args(["run", "dev:webui"])
            .current_dir("webui")
            .status()
            .map_err(|e| ApicentricError::runtime_error(
                format!("Failed to launch Web UI: {}", e),
                Some("Ensure you have run 'npm install' in the 'webui' directory")
            ));

        if let Err(e) = status {
            eprintln!("{}", e);
        }
    });

    println!("🚀 Starting Apicentric Web UI...");
    println!("- Backend API (GUI application) is starting...");
    println!("- Frontend (Next.js) is starting on http://localhost:9002");
    println!("- Please wait for both components to initialize.");

    // Wait for both threads to complete
    backend_handle.join().unwrap();
    frontend_handle.join().unwrap();

    Ok(())
}
