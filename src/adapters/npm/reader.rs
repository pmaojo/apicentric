use crate::errors::{PulseError, PulseResult};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{NpmScriptTemplate, NpmSetupStatus};

/// Handles reading and detection logic for npm integration
#[derive(Debug, Clone)]
pub struct NpmReader {
    pub project_root: PathBuf,
    pub package_json_path: PathBuf,
}

impl NpmReader {
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            package_json_path: project_root.join("package.json"),
        }
    }

    /// Read and parse package.json
    pub fn read_package_json(&self) -> PulseResult<Value> {
        let content = fs::read_to_string(&self.package_json_path).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read package.json: {}", e),
                Some("Check that package.json exists and is readable"),
            )
        })?;

        serde_json::from_str(&content).map_err(|e| {
            PulseError::config_error(
                format!("Invalid JSON in package.json: {}", e),
                Some("Fix JSON syntax errors in package.json"),
            )
        })
    }

    /// Resolve the path to the mockforge binary for npm scripts
    pub fn resolve_mockforge_binary_path(&self) -> PulseResult<String> {
        // Strategy 1: workspace with utils/mockforge
        let cargo_manifest = self.project_root.join("utils/mockforge/Cargo.toml");
        if cargo_manifest.exists() {
            return Ok("cargo run --manifest-path utils/mockforge/Cargo.toml --".to_string());
        }

        // Strategy 2: built binaries
        let target_debug = self.project_root.join("utils/mockforge/target/debug/mockforge");
        let target_release = self.project_root.join("utils/mockforge/target/release/mockforge");
        if target_release.exists() {
            return Ok("./utils/mockforge/target/release/mockforge".to_string());
        }
        if target_debug.exists() {
            return Ok("./utils/mockforge/target/debug/mockforge".to_string());
        }

        // Strategy 3: globally installed
        if let Ok(output) = Command::new("which").arg("mockforge").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok("mockforge".to_string());
                }
            }
        }

        // Strategy 4: cargo bin directory
        if let Ok(home) = std::env::var("HOME") {
            let cargo_bin = PathBuf::from(home).join(".cargo/bin/mockforge");
            if cargo_bin.exists() {
                return Ok("mockforge".to_string());
            }
        }

        Ok("cargo run --manifest-path utils/mockforge/Cargo.toml --".to_string())
    }

    /// Generate npm script templates based on the binary path
    pub fn generate_script_templates(&self, binary_path: &str) -> Vec<NpmScriptTemplate> {
        vec![
            NpmScriptTemplate {
                name: "mockforge".to_string(),
                command: binary_path.to_string(),
                description: "Run mockforge test runner".to_string(),
            },
            NpmScriptTemplate {
                name: "mockforge:watch".to_string(),
                command: format!("{} watch", binary_path),
                description: "Watch for changes and run tests".to_string(),
            },
        ]
    }

    /// Generate setup instructions based on current state
    pub fn generate_setup_instructions(
        &self,
        mockforge_script_exists: bool,
        mockforge_watch_script_exists: bool,
        templates: &[NpmScriptTemplate],
    ) -> Vec<String> {
        let mut instructions = Vec::new();

        if !mockforge_script_exists {
            if let Some(template) = templates.iter().find(|t| t.name == "mockforge") {
                instructions.push(format!(
                    "Add mockforge script: \"mockforge\": \"{}\"",
                    template.command
                ));
            }
        }

        if !mockforge_watch_script_exists {
            if let Some(template) = templates.iter().find(|t| t.name == "mockforge:watch") {
                instructions.push(format!(
                    "Add mockforge:watch script: \"mockforge:watch\": \"{}\"",
                    template.command
                ));
            }
        }

        instructions.push("Run 'npm run mockforge -- run' to execute all tests".to_string());
        instructions.push("Run 'npm run mockforge:watch' to watch for changes".to_string());

        instructions
    }

    /// Detect current npm script setup status
    pub fn detect_setup_status(&self) -> PulseResult<NpmSetupStatus> {
        let package_json_exists = self.package_json_path.exists();
        let mut mockforge_script_exists = false;
        let mut mockforge_watch_script_exists = false;
        let mut setup_instructions = Vec::new();

        if package_json_exists {
            if let Ok(package_json) = self.read_package_json() {
                if let Some(scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
                    mockforge_script_exists = scripts.contains_key("mockforge");
                    mockforge_watch_script_exists = scripts.contains_key("mockforge:watch");
                }
            }
        } else {
            setup_instructions.push("Create a package.json file in your project root".to_string());
        }

        let mockforge_binary_path = self.resolve_mockforge_binary_path()?;
        let recommended_scripts = self.generate_script_templates(&mockforge_binary_path);

        if !mockforge_script_exists || !mockforge_watch_script_exists {
            setup_instructions.extend(self.generate_setup_instructions(
                mockforge_script_exists,
                mockforge_watch_script_exists,
                &recommended_scripts,
            ));
        }

        Ok(NpmSetupStatus {
            package_json_exists,
            mockforge_script_exists,
            mockforge_watch_script_exists,
            mockforge_binary_path: Some(mockforge_binary_path),
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
