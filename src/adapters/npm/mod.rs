//! Provides an integration with the Node Package Manager (NPM).
//!
//! This module provides a way to detect the status of the NPM setup in a
//! project, and to set up the necessary scripts for running `apicentric`
//! commands.

use crate::errors::{ApicentricError, ApicentricResult};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A template for an NPM script.
#[derive(Debug, Clone)]
pub struct NpmScriptTemplate {
    /// The name of the script.
    pub name: String,
    /// The command that the script should run.
    pub command: String,
    /// A description of the script.
    pub description: String,
}

/// The status of the NPM setup in a project.
#[derive(Debug, Clone)]
pub struct NpmSetupStatus {
    /// Whether a `package.json` file exists in the project.
    pub package_json_exists: bool,
    /// Whether an `apicentric` script exists in the `package.json` file.
    pub apicentric_script_exists: bool,
    /// Whether an `apicentric:watch` script exists in the `package.json` file.
    pub apicentric_watch_script_exists: bool,
    /// The path to the `apicentric` binary.
    pub apicentric_binary_path: Option<String>,
    /// The recommended scripts to add to the `package.json` file.
    pub recommended_scripts: Vec<NpmScriptTemplate>,
    /// The instructions for setting up the scripts manually.
    pub setup_instructions: Vec<String>,
}

pub mod reader;
pub mod writer;

pub use reader::NpmReader;
pub use writer::NpmWriter;

/// A trait for integrating with NPM.
pub trait NpmIntegrator {
    /// Detects the status of the NPM setup in a project.
    fn detect_setup_status(&self) -> ApicentricResult<NpmSetupStatus>;
    /// Sets up the necessary scripts for running `apicentric` commands.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to overwrite existing scripts.
    fn setup_scripts(&self, force: bool) -> ApicentricResult<()>;
}

/// An integration with NPM that delegates to reader and writer components.
#[derive(Debug, Clone)]
pub struct NpmIntegration {
    project_root: PathBuf,
}

impl NpmIntegration {
    /// Creates a new `NpmIntegration`.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project.
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    fn reader(&self) -> NpmReader {
        NpmReader::new(&self.project_root)
    }

    fn writer(&self) -> NpmWriter {
        NpmWriter::new(&self.project_root)
    }

    /// Detects the current NPM setup status.
    pub fn detect_setup_status(&self) -> ApicentricResult<NpmSetupStatus> {
        self.reader().detect_setup_status()
    }

    /// Automatically sets up the NPM scripts.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to overwrite existing scripts.
    pub fn setup_scripts(&self, force: bool) -> ApicentricResult<()> {
        self.writer().setup_scripts(force)
    }

    /// Reads the `package.json` file.
    pub fn read_package_json(&self) -> ApicentricResult<Value> {
        self.reader().read_package_json()
    }

    /// Validates the NPM setup.
    pub fn validate_npm_setup(&self) -> ApicentricResult<bool> {
        let status = self.detect_setup_status()?;

        if !status.package_json_exists {
            println!("❌ package.json not found");
            return Ok(false);
        }
        if !status.apicentric_script_exists {
            println!("❌ 'apicentric' script not found in package.json");
            return Ok(false);
        }
        if !status.apicentric_watch_script_exists {
            println!("❌ 'apicentric:watch' script not found in package.json");
            return Ok(false);
        }

        println!("✅ NPM scripts are properly configured");
        Ok(true)
    }

    /// Prints the instructions for setting up the scripts manually.
    pub fn print_setup_instructions(&self) -> ApicentricResult<()> {
        let status = self.detect_setup_status()?;

        println!("📋 apicentric NPM Script Setup Instructions");
        println!("=====================================");
        println!("📁 Project Root: {}", self.project_root.display());

        if !status.package_json_exists {
            println!("\n❌ package.json not found in project root");
            println!("   Run 'npm init' or 'yarn init' to create one in the project root");
            return Ok(());
        }

        if status.apicentric_script_exists && status.apicentric_watch_script_exists {
            println!("\n✅ All apicentric scripts are already configured!");
            println!("   You can run:");
            println!("   - npm run apicentric -- run");
            println!("   - npm run apicentric:watch");
            return Ok(());
        }

        println!("\n📝 Add the following scripts to your package.json:");
        println!("```json");
        println!("{{");
        println!("  \"scripts\": {{");
        for (i, template) in status.recommended_scripts.iter().enumerate() {
            let comma = if i < status.recommended_scripts.len() - 1 { "," } else { "" };
            println!(
                "    \"{}\": \"{}\"{}  // {}",
                template.name, template.command, comma, template.description
            );
        }
        println!("  }}");
        println!("}}");
        println!("```");

        println!("\n🚀 After adding the scripts, you can run:");
        println!("   - npm run apicentric -- run        # Run all tests");
        println!("   - npm run apicentric -- watch      # Watch for changes");
        println!("   - npm run apicentric:watch         # Watch for changes (shortcut)");

        if let Some(binary_path) = &status.apicentric_binary_path {
            println!("\n🔧 Binary path detected: {}", binary_path);
        }

        Ok(())
    }

    /// Shows usage examples for the NPM scripts.
    pub fn show_usage_examples(&self) -> ApicentricResult<()> {
        println!("📚 apicentric NPM Integration Usage Examples");
        println!("======================================");
        println!();
        println!("📁 Working Directory: {}", self.project_root.display());
        println!(
            "   NPM scripts automatically run from the project root where package.json is located",
        );
        println!();
        println!("🚀 Basic Usage:");
        println!("   npm run apicentric -- run              # Run all tests");
        println!("   npm run apicentric -- watch            # Watch for changes");
        println!("   npm run apicentric:watch               # Watch for changes (shortcut)");
        println!();
        println!("⚙️ Advanced Usage:");
        println!("   npm run apicentric -- run --workers 8  # Run with 8 parallel workers");
        println!("   npm run apicentric -- watch --retries 5 # Watch with 5 retries");
        println!("   npm run apicentric -- --mode ci run    # Run in CI mode");
        println!("   npm run apicentric -- --dry-run run    # Show what would be executed");
        println!("   npm run apicentric -- --verbose watch  # Verbose output");
        println!();
        println!("🔧 Configuration:");
        println!("   npm run apicentric -- --config custom.json run  # Use custom config");
        println!("   npm run apicentric -- --help                    # Show all options");
        println!();
        println!("📋 Setup Commands:");
        println!("   cargo run --manifest-path utils/apicentric/Cargo.toml -- setup-npm");
        println!("   cargo run --manifest-path utils/apicentric/Cargo.toml -- setup-npm --instructions-only");
        println!("   cargo run --manifest-path utils/apicentric/Cargo.toml -- setup-npm --force");
        println!("   cargo run --manifest-path utils/apicentric/Cargo.toml -- setup-npm --test");
        println!();
        println!("💡 Note: All commands run from the project root directory automatically");
        Ok(())
    }

    /// Tests the NPM script execution.
    pub fn test_npm_scripts(&self) -> ApicentricResult<()> {
        let status = self.detect_setup_status()?;
        if !status.apicentric_script_exists {
            return Err(ApicentricError::config_error(
                "apicentric script not configured",
                Some("Run apicentric setup-npm to configure npm scripts"),
            ));
        }

        println!("🧪 Testing npm script execution...");
        println!("   Working directory: {}", self.project_root.display());

        println!("   Testing 'npm run apicentric -- --help'");
        let output = Command::new("npm")
            .args(["run", "apicentric", "--", "--help"])
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| {
                ApicentricError::fs_error(
                    format!(
                        "Failed to execute npm run apicentric from {}: {}",
                        self.project_root.display(),
                        e
                    ),
                    Some("Check that npm is installed and apicentric script is configured correctly"),
                )
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApicentricError::test_error(
                format!("npm run apicentric failed: {}", stderr),
                Some("Check apicentric binary path and configuration"),
            ));
        }
        println!("   ✅ 'npm run apicentric' works correctly");

        if status.apicentric_watch_script_exists {
            println!("   Testing 'npm run apicentric:watch -- --help'");
            let output = Command::new("npm")
                .args(["run", "apicentric:watch", "--", "--help"])
                .current_dir(&self.project_root)
                .output()
                .map_err(|e| {
                    ApicentricError::fs_error(
                        format!(
                            "Failed to execute npm run apicentric:watch from {}: {}",
                            self.project_root.display(),
                            e
                        ),
                        Some(
                            "Check that npm is installed and apicentric:watch script is configured correctly",
                        ),
                    )
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ApicentricError::test_error(
                    format!("npm run apicentric:watch failed: {}", stderr),
                    Some("Check apicentric binary path and configuration"),
                ));
            }
            println!("   ✅ 'npm run apicentric:watch' works correctly");
        }

        println!("✅ All npm scripts are working correctly");
        Ok(())
    }

    /// Resolves the path to the `apicentric` binary.
    pub fn resolve_apicentric_binary_path(&self) -> ApicentricResult<String> {
        self.reader().resolve_apicentric_binary_path()
    }
}

impl NpmIntegrator for NpmIntegration {
    fn detect_setup_status(&self) -> ApicentricResult<NpmSetupStatus> {
        self.reader().detect_setup_status()
    }

    fn setup_scripts(&self, force: bool) -> ApicentricResult<()> {
        self.writer().setup_scripts(force)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn integration_detects_missing_file() {
        let tmp = TempDir::new().unwrap();
        let npm = NpmIntegration::new(tmp.path());
        let status = npm.detect_setup_status().unwrap();
        assert!(!status.package_json_exists);
    }

    #[test]
    fn integration_sets_up_scripts() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package.json"), "{\"name\":\"t\", \"version\":\"1\"}").unwrap();
        let npm = NpmIntegration::new(tmp.path());
        npm.setup_scripts(false).unwrap();
        let json = npm.read_package_json().unwrap();
        let scripts = json["scripts"].as_object().unwrap();
        assert!(scripts.contains_key("apicentric"));
        assert!(scripts.contains_key("apicentric:watch"));
    }
}
