//! Handles reading and detection logic for NPM integration.
//!
//! This module provides the `NpmReader` struct, which is responsible for
//! reading and parsing the `package.json` file, resolving the path to the
//! `apicentric` binary, and detecting the current NPM script setup status.

use crate::errors::{ApicentricError, ApicentricResult};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{NpmScriptTemplate, NpmSetupStatus};

/// Handles reading and detection logic for npm integration.
#[derive(Debug, Clone)]
pub struct NpmReader {
    /// The root directory of the project.
    pub project_root: PathBuf,
    /// The path to the `package.json` file.
    pub package_json_path: PathBuf,
}

impl NpmReader {
    /// Creates a new `NpmReader`.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project.
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            package_json_path: project_root.join("package.json"),
        }
    }

    /// Reads and parses the `package.json` file.
    pub fn read_package_json(&self) -> ApicentricResult<Value> {
        let content = fs::read_to_string(&self.package_json_path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot read package.json: {}", e),
                Some("Check that package.json exists and is readable"),
            )
        })?;

        serde_json::from_str(&content).map_err(|e| {
            ApicentricError::config_error(
                format!("Invalid JSON in package.json: {}", e),
                Some("Fix JSON syntax errors in package.json"),
            )
        })
    }

    /// Resolves the path to the `apicentric` binary for NPM scripts.
    pub fn resolve_apicentric_binary_path(&self) -> ApicentricResult<String> {
        // Strategy 1: workspace with utils/apicentric
        let cargo_manifest = self.project_root.join("utils/apicentric/Cargo.toml");
        if cargo_manifest.exists() {
            return Ok("cargo run --manifest-path utils/apicentric/Cargo.toml --".to_string());
        }

        // Strategy 2: built binaries
        let target_debug = self.project_root.join("utils/apicentric/target/debug/apicentric");
        let target_release = self.project_root.join("utils/apicentric/target/release/apicentric");
        if target_release.exists() {
            return Ok("./utils/apicentric/target/release/apicentric".to_string());
        }
        if target_debug.exists() {
            return Ok("./utils/apicentric/target/debug/apicentric".to_string());
        }

        // Strategy 3: globally installed
        if let Ok(output) = Command::new("which").arg("apicentric").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok("apicentric".to_string());
                }
            }
        }

        // Strategy 4: cargo bin directory
        if let Ok(home) = std::env::var("HOME") {
            let cargo_bin = PathBuf::from(home).join(".cargo/bin/apicentric");
            if cargo_bin.exists() {
                return Ok("apicentric".to_string());
            }
        }

        Ok("cargo run --manifest-path utils/apicentric/Cargo.toml --".to_string())
    }

    /// Generates NPM script templates based on the binary path.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - The path to the `apicentric` binary.
    pub fn generate_script_templates(&self, binary_path: &str) -> Vec<NpmScriptTemplate> {
        vec![
            NpmScriptTemplate {
                name: "apicentric".to_string(),
                command: binary_path.to_string(),
                description: "Run apicentric test runner".to_string(),
            },
            NpmScriptTemplate {
                name: "apicentric:watch".to_string(),
                command: format!("{} watch", binary_path),
                description: "Watch for changes and run tests".to_string(),
            },
        ]
    }

    /// Generates setup instructions based on the current state.
    ///
    /// # Arguments
    ///
    /// * `apicentric_script_exists` - Whether the `apicentric` script exists.
    /// * `apicentric_watch_script_exists` - Whether the `apicentric:watch` script exists.
    /// * `templates` - The recommended script templates.
    pub fn generate_setup_instructions(
        &self,
        apicentric_script_exists: bool,
        apicentric_watch_script_exists: bool,
        templates: &[NpmScriptTemplate],
    ) -> Vec<String> {
        let mut instructions = Vec::new();

        if !apicentric_script_exists {
            if let Some(template) = templates.iter().find(|t| t.name == "apicentric") {
                instructions.push(format!(
                    "Add apicentric script: \"apicentric\": \"{}\"",
                    template.command
                ));
            }
        }

        if !apicentric_watch_script_exists {
            if let Some(template) = templates.iter().find(|t| t.name == "apicentric:watch") {
                instructions.push(format!(
                    "Add apicentric:watch script: \"apicentric:watch\": \"{}\"",
                    template.command
                ));
            }
        }

        instructions.push("Run 'npm run apicentric -- run' to execute all tests".to_string());
        instructions.push("Run 'npm run apicentric:watch' to watch for changes".to_string());

        instructions
    }

    /// Detects the current NPM script setup status.
    pub fn detect_setup_status(&self) -> ApicentricResult<NpmSetupStatus> {
        let package_json_exists = self.package_json_path.exists();
        let mut apicentric_script_exists = false;
        let mut apicentric_watch_script_exists = false;
        let mut setup_instructions = Vec::new();

        if package_json_exists {
            if let Ok(package_json) = self.read_package_json() {
                if let Some(scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
                    apicentric_script_exists = scripts.contains_key("apicentric");
                    apicentric_watch_script_exists = scripts.contains_key("apicentric:watch");
                }
            }
        } else {
            setup_instructions.push("Create a package.json file in your project root".to_string());
        }

        let apicentric_binary_path = self.resolve_apicentric_binary_path()?;
        let recommended_scripts = self.generate_script_templates(&apicentric_binary_path);

        if !apicentric_script_exists || !apicentric_watch_script_exists {
            setup_instructions.extend(self.generate_setup_instructions(
                apicentric_script_exists,
                apicentric_watch_script_exists,
                &recommended_scripts,
            ));
        }

        Ok(NpmSetupStatus {
            package_json_exists,
            apicentric_script_exists,
            apicentric_watch_script_exists,
            apicentric_binary_path: Some(apicentric_binary_path),
            recommended_scripts,
            setup_instructions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn reads_valid_package_json() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{\"name\":\"test\"}").unwrap();
        let reader = NpmReader::new(temp_dir.path());
        let json = reader.read_package_json().unwrap();
        assert_eq!(json["name"], "test");
    }

    #[test]
    fn detect_status_without_file() {
        let temp_dir = TempDir::new().unwrap();
        let reader = NpmReader::new(temp_dir.path());
        let status = reader.detect_setup_status().unwrap();
        assert!(!status.package_json_exists);
    }
}
