//! Handles writing and patching logic for NPM integration.
//!
//! This module provides the `NpmWriter` struct, which is responsible for
//! adding and updating scripts in a `package.json` file.

use crate::errors::{ApicentricError, ApicentricResult};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

use super::reader::NpmReader;

/// Handles writing and patching logic for npm integration.
#[derive(Debug, Clone)]
pub struct NpmWriter {
    reader: NpmReader,
}

impl NpmWriter {
    /// Creates a new `NpmWriter`.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project.
    pub fn new(project_root: &Path) -> Self {
        Self {
            reader: NpmReader::new(project_root),
        }
    }

    /// Automatically adds `apicentric` scripts to `package.json`.
    ///
    /// # Arguments
    ///
    /// * `force` - Whether to overwrite existing scripts.
    pub fn setup_scripts(&self, force: bool) -> ApicentricResult<()> {
        if !self.reader.package_json_path.exists() {
            return Err(ApicentricError::fs_error(
                "package.json not found",
                Some("Create a package.json file first using 'npm init' or 'yarn init'"),
            ));
        }

        let mut package_json = self.reader.read_package_json()?;
        let apicentric_binary_path = self.reader.resolve_apicentric_binary_path()?;
        let script_templates = self
            .reader
            .generate_script_templates(&apicentric_binary_path);

        if !package_json.is_object() {
            return Err(ApicentricError::config_error(
                "package.json is not a valid JSON object",
                Some("Fix the package.json file structure"),
            ));
        }

        let package_obj = package_json.as_object_mut().unwrap();
        if !package_obj.contains_key("scripts") {
            package_obj.insert("scripts".to_string(), Value::Object(Map::new()));
        }

        let scripts = package_obj
            .get_mut("scripts")
            .unwrap()
            .as_object_mut()
            .ok_or_else(|| {
                ApicentricError::config_error(
                    "scripts field in package.json is not an object",
                    Some("Fix the scripts field in package.json to be an object"),
                )
            })?;

        let mut added_scripts = Vec::new();
        let mut skipped_scripts = Vec::new();

        for template in script_templates {
            let script_exists = scripts.contains_key(&template.name);

            if script_exists && !force {
                skipped_scripts.push(template.name.clone());
                continue;
            }

            scripts.insert(template.name.clone(), Value::String(template.command));
            added_scripts.push(template.name);
        }

        self.write_package_json(&package_json)?;

        if !added_scripts.is_empty() {
            println!("✅ Added npm scripts: {}", added_scripts.join(", "));
        }
        if !skipped_scripts.is_empty() {
            println!(
                "⏭️ Skipped existing scripts: {} (use --force to overwrite)",
                skipped_scripts.join(", ")
            );
        }
        if added_scripts.is_empty() && skipped_scripts.is_empty() {
            println!("ℹ️ All apicentric scripts are already configured");
        }

        Ok(())
    }

    /// Writes the `package.json` file with proper formatting.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The `package.json` data to write.
    pub fn write_package_json(&self, package_json: &Value) -> ApicentricResult<()> {
        let content = serde_json::to_string_pretty(package_json).map_err(|e| {
            ApicentricError::config_error(
                format!("Cannot serialize package.json: {}", e),
                None::<String>,
            )
        })?;

        fs::write(&self.reader.package_json_path, content).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot write package.json: {}", e),
                Some("Check write permissions for package.json"),
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_package_json(dir: &Path) {
        let content = r#"{
  "name": "test",
  "version": "1.0.0",
  "scripts": {"build": "echo build"}
}"#;
        fs::write(dir.join("package.json"), content).unwrap();
    }

    #[test]
    fn adds_scripts_to_package_json() {
        let temp_dir = TempDir::new().unwrap();
        create_package_json(temp_dir.path());
        let writer = NpmWriter::new(temp_dir.path());
        writer.setup_scripts(false).unwrap();
        let reader = NpmReader::new(temp_dir.path());
        let json = reader.read_package_json().unwrap();
        let scripts = json["scripts"].as_object().unwrap();
        assert!(scripts.contains_key("apicentric"));
        assert!(scripts.contains_key("apicentric:watch"));
    }
}
