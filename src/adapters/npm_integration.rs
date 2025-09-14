use crate::errors::{PulseError, PulseResult};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// NPM script integration adapter for MockForge
pub struct NpmIntegration {
    project_root: PathBuf,
    package_json_path: PathBuf,
}

/// Template for npm scripts that can be added to package.json
#[derive(Debug, Clone)]
pub struct NpmScriptTemplate {
    pub name: String,
    pub command: String,
    pub description: String,
}

/// Status of npm script setup in the project
#[derive(Debug, Clone)]
pub struct NpmSetupStatus {
    pub package_json_exists: bool,
    pub mockforge_script_exists: bool,
    pub mockforge_watch_script_exists: bool,
    pub mockforge_binary_path: Option<String>,
    pub recommended_scripts: Vec<NpmScriptTemplate>,
    pub setup_instructions: Vec<String>,
}

impl NpmIntegration {
    /// Create a new npm integration adapter
    pub fn new(project_root: &Path) -> Self {
        let package_json_path = project_root.join("package.json");

        Self {
            project_root: project_root.to_path_buf(),
            package_json_path,
        }
    }

    /// Detect current npm script setup status
    pub fn detect_setup_status(&self) -> PulseResult<NpmSetupStatus> {
        let package_json_exists = self.package_json_path.exists();
        let mut mockforge_script_exists = false;
        let mut mockforge_watch_script_exists = false;
        let mut setup_instructions = Vec::new();

        // Check existing scripts if package.json exists
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

        // Resolve mockforge binary path
        let mockforge_binary_path = self.resolve_mockforge_binary_path()?;

        // Generate recommended scripts
        let recommended_scripts = self.generate_script_templates(&mockforge_binary_path);

        // Generate setup instructions
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

    /// Automatically add mockforge scripts to package.json
    pub fn setup_scripts(&self, force: bool) -> PulseResult<()> {
        if !self.package_json_path.exists() {
            return Err(PulseError::fs_error(
                "package.json not found",
                Some("Create a package.json file first using 'npm init' or 'yarn init'"),
            ));
        }

        let mut package_json = self.read_package_json()?;
        let mockforge_binary_path = self.resolve_mockforge_binary_path()?;
        let script_templates = self.generate_script_templates(&mockforge_binary_path);

        // Get or create scripts section
        if !package_json.is_object() {
            return Err(PulseError::config_error(
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
                PulseError::config_error(
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

        // Save updated package.json
        self.write_package_json(&package_json)?;

        // Report results
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
            println!("ℹ️ All mockforge scripts are already configured");
        }

        Ok(())
    }

    /// Generate setup instructions for manual configuration
    pub fn print_setup_instructions(&self) -> PulseResult<()> {
        let status = self.detect_setup_status()?;

        println!("📋 MockForge NPM Script Setup Instructions");
        println!("=====================================");
        println!("📁 Project Root: {}", self.project_root.display());

        if !status.package_json_exists {
            println!("\n❌ package.json not found in project root");
            println!("   Run 'npm init' or 'yarn init' to create one in the project root");
            return Ok(());
        }

        if status.mockforge_script_exists && status.mockforge_watch_script_exists {
            println!("\n✅ All mockforge scripts are already configured!");
            println!("   You can run:");
            println!("   - npm run mockforge -- run");
            println!("   - npm run mockforge:watch");
            return Ok(());
        }

        println!("\n📝 Add the following scripts to your package.json:");
        println!("```json");
        println!("{{");
        println!("  \"scripts\": {{");

        for (i, template) in status.recommended_scripts.iter().enumerate() {
            let comma = if i < status.recommended_scripts.len() - 1 {
                ","
            } else {
                ""
            };
            println!(
                "    \"{}\": \"{}\"{}  // {}",
                template.name, template.command, comma, template.description
            );
        }

        println!("  }}");
        println!("}}");
        println!("```");

        println!("\n🚀 After adding the scripts, you can run:");
        println!("   - npm run mockforge -- run        # Run all tests");
        println!("   - npm run mockforge -- watch      # Watch for changes");
        println!("   - npm run mockforge:watch         # Watch for changes (shortcut)");

        if let Some(binary_path) = &status.mockforge_binary_path {
            println!("\n🔧 Binary path detected: {}", binary_path);
        } else {
            println!("\n⚠️ Could not detect mockforge binary path");
            println!(
                "   Make sure mockforge is built: cargo build --manifest-path utils/mockforge/Cargo.toml"
            );
        }

        Ok(())
    }

    /// Resolve the path to the mockforge binary for npm scripts
    pub fn resolve_mockforge_binary_path(&self) -> PulseResult<String> {
        // Try different strategies to find the mockforge binary

        // Strategy 1: Check if we're in a workspace with utils/mockforge
        let cargo_manifest = self.project_root.join("utils/mockforge/Cargo.toml");
        if cargo_manifest.exists() {
            return Ok("cargo run --manifest-path utils/mockforge/Cargo.toml --".to_string());
        }

        // Strategy 2: Check if mockforge is built in target directory
        let target_debug = self.project_root.join("utils/mockforge/target/debug/mockforge");
        let target_release = self.project_root.join("utils/mockforge/target/release/mockforge");

        if target_release.exists() {
            return Ok("./utils/mockforge/target/release/mockforge".to_string());
        }

        if target_debug.exists() {
            return Ok("./utils/mockforge/target/debug/mockforge".to_string());
        }

        // Strategy 3: Check if mockforge is installed globally
        if let Ok(output) = Command::new("which").arg("mockforge").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok("mockforge".to_string());
                }
            }
        }

        // Strategy 4: Check common cargo install locations
        if let Ok(home) = std::env::var("HOME") {
            let cargo_bin = PathBuf::from(home).join(".cargo/bin/mockforge");
            if cargo_bin.exists() {
                return Ok("mockforge".to_string());
            }
        }

        // Fallback: Use cargo run command (most reliable)
        Ok("cargo run --manifest-path utils/mockforge/Cargo.toml --".to_string())
    }

    /// Generate npm script templates based on the binary path
    fn generate_script_templates(&self, binary_path: &str) -> Vec<NpmScriptTemplate> {
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
    fn generate_setup_instructions(
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

    /// Write package.json with proper formatting
    fn write_package_json(&self, package_json: &Value) -> PulseResult<()> {
        let content = serde_json::to_string_pretty(package_json).map_err(|e| {
            PulseError::config_error(
                format!("Cannot serialize package.json: {}", e),
                None::<String>,
            )
        })?;

        fs::write(&self.package_json_path, content).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot write package.json: {}", e),
                Some("Check write permissions for package.json"),
            )
        })?;

        Ok(())
    }

    /// Check if npm scripts are properly configured
    pub fn validate_npm_setup(&self) -> PulseResult<bool> {
        let status = self.detect_setup_status()?;

        if !status.package_json_exists {
            println!("❌ package.json not found");
            return Ok(false);
        }

        if !status.mockforge_script_exists {
            println!("❌ 'mockforge' script not found in package.json");
            return Ok(false);
        }

        if !status.mockforge_watch_script_exists {
            println!("❌ 'mockforge:watch' script not found in package.json");
            return Ok(false);
        }

        println!("✅ NPM scripts are properly configured");
        Ok(true)
    }

    /// Show usage examples for npm scripts
    pub fn show_usage_examples(&self) -> PulseResult<()> {
        println!("📚 MockForge NPM Integration Usage Examples");
        println!("======================================");
        println!();
        println!("📁 Working Directory: {}", self.project_root.display());
        println!(
            "   NPM scripts automatically run from the project root where package.json is located"
        );
        println!();
        println!("🚀 Basic Usage:");
        println!("   npm run mockforge -- run              # Run all tests");
        println!("   npm run mockforge -- watch            # Watch for changes");
        println!("   npm run mockforge:watch               # Watch for changes (shortcut)");
        println!();
        println!("⚙️ Advanced Usage:");
        println!("   npm run mockforge -- run --workers 8  # Run with 8 parallel workers");
        println!("   npm run mockforge -- watch --retries 5 # Watch with 5 retries");
        println!("   npm run mockforge -- --mode ci run    # Run in CI mode");
        println!("   npm run mockforge -- --dry-run run    # Show what would be executed");
        println!("   npm run mockforge -- --verbose watch  # Verbose output");
        println!();
        println!("🔧 Configuration:");
        println!("   npm run mockforge -- --config custom.json run  # Use custom config");
        println!("   npm run mockforge -- --help                    # Show all options");
        println!();
        println!("📋 Setup Commands:");
        println!("   cargo run --manifest-path utils/mockforge/Cargo.toml -- setup-npm");
        println!(
            "   cargo run --manifest-path utils/mockforge/Cargo.toml -- setup-npm --instructions-only"
        );
        println!("   cargo run --manifest-path utils/mockforge/Cargo.toml -- setup-npm --force");
        println!("   cargo run --manifest-path utils/mockforge/Cargo.toml -- setup-npm --test");
        println!();
        println!("💡 Note: All commands run from the project root directory automatically");

        Ok(())
    }

    /// Test npm script execution (dry run)
    pub fn test_npm_scripts(&self) -> PulseResult<()> {
        let status = self.detect_setup_status()?;

        if !status.mockforge_script_exists {
            return Err(PulseError::config_error(
                "mockforge script not configured",
                Some("Run mockforge setup-npm to configure npm scripts"),
            ));
        }

        println!("🧪 Testing npm script execution...");
        println!("   Working directory: {}", self.project_root.display());

        // Test mockforge script with --help
        println!("   Testing 'npm run mockforge -- --help'");
        let output = Command::new("npm")
            .args(&["run", "mockforge", "--", "--help"])
            .current_dir(&self.project_root)
            .output()
            .map_err(|e| {
                PulseError::fs_error(
                    format!(
                        "Failed to execute npm run mockforge from {}: {}",
                        self.project_root.display(),
                        e
                    ),
                    Some("Check that npm is installed and mockforge script is configured correctly"),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PulseError::test_error(
                format!("npm run mockforge failed: {}", stderr),
                Some("Check mockforge binary path and configuration"),
            ));
        }

        println!("   ✅ 'npm run mockforge' works correctly");

        // Test mockforge:watch script with --help if it exists
        if status.mockforge_watch_script_exists {
            println!("   Testing 'npm run mockforge:watch -- --help'");
            let output = Command::new("npm")
                .args(&["run", "mockforge:watch", "--", "--help"])
                .current_dir(&self.project_root)
                .output()
                .map_err(|e| PulseError::fs_error(
                    format!("Failed to execute npm run mockforge:watch from {}: {}", self.project_root.display(), e),
                    Some("Check that npm is installed and mockforge:watch script is configured correctly")
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(PulseError::test_error(
                    format!("npm run mockforge:watch failed: {}", stderr),
                    Some("Check mockforge binary path and configuration"),
                ));
            }

            println!("   ✅ 'npm run mockforge:watch' works correctly");
        }

        println!("✅ All npm scripts are working correctly");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_package_json(dir: &Path, scripts: Option<&str>) -> std::io::Result<()> {
        let scripts_section = scripts.unwrap_or(r#""build": "echo build""#);
        let content = format!(
            r#"{{
  "name": "test-project",
  "version": "1.0.0",
  "scripts": {{
    {}
  }}
}}"#,
            scripts_section
        );

        fs::write(dir.join("package.json"), content)
    }

    fn create_invalid_package_json(dir: &Path) -> std::io::Result<()> {
        let content = r#"{ "name": "test", invalid json }"#;
        fs::write(dir.join("package.json"), content)
    }

    #[test]
    fn test_npm_integration_creation() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());

        assert_eq!(npm_integration.project_root, temp_dir.path());
        assert_eq!(
            npm_integration.package_json_path,
            temp_dir.path().join("package.json")
        );
    }

    #[test]
    fn test_detect_setup_status_no_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());

        let status = npm_integration.detect_setup_status().unwrap();

        assert!(!status.package_json_exists);
        assert!(!status.mockforge_script_exists);
        assert!(!status.mockforge_watch_script_exists);
        assert!(!status.setup_instructions.is_empty());
        assert!(status
            .setup_instructions
            .iter()
            .any(|s| s.contains("Create a package.json")));
    }

    #[test]
    fn test_detect_setup_status_with_package_json() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path(), None).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let status = npm_integration.detect_setup_status().unwrap();

        assert!(status.package_json_exists);
        assert!(!status.mockforge_script_exists);
        assert!(!status.mockforge_watch_script_exists);
        assert!(status.mockforge_binary_path.is_some());
        assert_eq!(status.recommended_scripts.len(), 2);
        assert!(!status.setup_instructions.is_empty());
    }

    #[test]
    fn test_detect_setup_status_with_existing_scripts() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(
            temp_dir.path(),
            Some(r#""mockforge": "cargo run --", "mockforge:watch": "cargo run -- watch""#),
        )
        .unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let status = npm_integration.detect_setup_status().unwrap();

        assert!(status.package_json_exists);
        assert!(status.mockforge_script_exists);
        assert!(status.mockforge_watch_script_exists);
        // When everything is configured, setup instructions might be empty or contain usage info
        // This is acceptable behavior
    }

    #[test]
    fn test_detect_setup_status_partial_scripts() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(
            temp_dir.path(),
            Some(r#""mockforge": "cargo run --""#), // Only mockforge, not mockforge:watch
        )
        .unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let status = npm_integration.detect_setup_status().unwrap();

        assert!(status.package_json_exists);
        assert!(status.mockforge_script_exists);
        assert!(!status.mockforge_watch_script_exists);
        assert!(!status.setup_instructions.is_empty());
    }

    #[test]
    fn test_detect_setup_status_invalid_package_json() {
        let temp_dir = TempDir::new().unwrap();
        create_invalid_package_json(temp_dir.path()).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let status = npm_integration.detect_setup_status().unwrap();

        // Should still detect that package.json exists, but scripts won't be detected
        assert!(status.package_json_exists);
        assert!(!status.mockforge_script_exists);
        assert!(!status.mockforge_watch_script_exists);
    }

    #[test]
    fn test_setup_scripts() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path(), None).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        npm_integration.setup_scripts(false).unwrap();

        // Verify scripts were added
        let status = npm_integration.detect_setup_status().unwrap();
        assert!(status.mockforge_script_exists);
        assert!(status.mockforge_watch_script_exists);

        // Verify the actual content
        let package_json = npm_integration.read_package_json().unwrap();
        let scripts = package_json["scripts"].as_object().unwrap();
        assert!(scripts.contains_key("mockforge"));
        assert!(scripts.contains_key("mockforge:watch"));
        assert!(scripts["mockforge:watch"].as_str().unwrap().contains("watch"));
    }

    #[test]
    fn test_setup_scripts_no_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());

        let result = npm_integration.setup_scripts(false);
        assert!(result.is_err());

        if let Err(PulseError::FileSystem {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("package.json not found"));
            assert!(suggestion.as_ref().unwrap().contains("npm init"));
        } else {
            panic!("Expected filesystem error");
        }
    }

    #[test]
    fn test_setup_scripts_skip_existing() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path(), Some(r#""mockforge": "existing-command""#)).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        npm_integration.setup_scripts(false).unwrap();

        // Verify existing script was not overwritten
        let package_json = npm_integration.read_package_json().unwrap();
        let scripts = package_json["scripts"].as_object().unwrap();
        assert_eq!(scripts["mockforge"].as_str().unwrap(), "existing-command");

        // But mockforge:watch should be added
        assert!(scripts.contains_key("mockforge:watch"));
    }

    #[test]
    fn test_setup_scripts_force_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(
            temp_dir.path(),
            Some(r#""mockforge": "existing-command", "mockforge:watch": "existing-watch""#),
        )
        .unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        npm_integration.setup_scripts(true).unwrap();

        // Verify existing scripts were overwritten
        let package_json = npm_integration.read_package_json().unwrap();
        let scripts = package_json["scripts"].as_object().unwrap();
        assert_ne!(scripts["mockforge"].as_str().unwrap(), "existing-command");
        assert_ne!(scripts["mockforge:watch"].as_str().unwrap(), "existing-watch");
    }

    #[test]
    fn test_setup_scripts_no_scripts_section() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{"name": "test-project", "version": "1.0.0"}"#;
        fs::write(temp_dir.path().join("package.json"), content).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        npm_integration.setup_scripts(false).unwrap();

        // Verify scripts section was created and populated
        let package_json = npm_integration.read_package_json().unwrap();
        let scripts = package_json["scripts"].as_object().unwrap();
        assert!(scripts.contains_key("mockforge"));
        assert!(scripts.contains_key("mockforge:watch"));
    }

    #[test]
    fn test_setup_scripts_invalid_scripts_section() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{"name": "test-project", "version": "1.0.0", "scripts": "not-an-object"}"#;
        fs::write(temp_dir.path().join("package.json"), content).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let result = npm_integration.setup_scripts(false);

        assert!(result.is_err());
        if let Err(PulseError::Configuration { message, .. }) = result {
            assert!(message.contains("scripts field"));
            assert!(message.contains("not an object"));
        } else {
            panic!("Expected configuration error");
        }
    }

    #[test]
    fn test_resolve_mockforge_binary_path_workspace() {
        let temp_dir = TempDir::new().unwrap();

        // Create utils/mockforge/Cargo.toml to simulate workspace structure
        let utils_dir = temp_dir.path().join("utils/mockforge");
        fs::create_dir_all(&utils_dir).unwrap();
        fs::write(utils_dir.join("Cargo.toml"), "[package]\nname = \"mockforge\"").unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let binary_path = npm_integration.resolve_mockforge_binary_path().unwrap();

        assert_eq!(
            binary_path,
            "cargo run --manifest-path utils/mockforge/Cargo.toml --"
        );
    }

    #[test]
    fn test_resolve_mockforge_binary_path_built_release() {
        let temp_dir = TempDir::new().unwrap();

        // Create target/release/mockforge binary
        let target_dir = temp_dir.path().join("utils/mockforge/target/release");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("mockforge"), "fake binary").unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let binary_path = npm_integration.resolve_mockforge_binary_path().unwrap();

        assert_eq!(binary_path, "./utils/mockforge/target/release/mockforge");
    }

    #[test]
    fn test_resolve_mockforge_binary_path_built_debug() {
        let temp_dir = TempDir::new().unwrap();

        // Create target/debug/mockforge binary (but not release)
        let target_dir = temp_dir.path().join("utils/mockforge/target/debug");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("mockforge"), "fake binary").unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let binary_path = npm_integration.resolve_mockforge_binary_path().unwrap();

        assert_eq!(binary_path, "./utils/mockforge/target/debug/mockforge");
    }

    #[test]
    fn test_resolve_mockforge_binary_path_fallback() {
        let temp_dir = TempDir::new().unwrap();
        // No special setup - should fall back to cargo run

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let binary_path = npm_integration.resolve_mockforge_binary_path().unwrap();

        assert_eq!(
            binary_path,
            "cargo run --manifest-path utils/mockforge/Cargo.toml --"
        );
    }

    #[test]
    fn test_generate_script_templates() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());

        let templates = npm_integration.generate_script_templates("test-binary");

        assert_eq!(templates.len(), 2);

        let mockforge_template = &templates[0];
        assert_eq!(mockforge_template.name, "mockforge");
        assert_eq!(mockforge_template.command, "test-binary");
        assert!(mockforge_template.description.contains("Run mockforge"));

        let watch_template = &templates[1];
        assert_eq!(watch_template.name, "mockforge:watch");
        assert_eq!(watch_template.command, "test-binary watch");
        assert!(watch_template.description.contains("Watch"));
    }

    #[test]
    fn test_generate_setup_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());
        let templates = npm_integration.generate_script_templates("test-binary");

        // Test when no scripts exist
        let instructions = npm_integration.generate_setup_instructions(false, false, &templates);
        assert!(instructions.len() >= 4); // Should have instructions for both scripts plus usage
        assert!(instructions.iter().any(|s| s.contains("mockforge script")));
        assert!(instructions
            .iter()
            .any(|s| s.contains("mockforge:watch script")));
        assert!(instructions.iter().any(|s| s.contains("npm run mockforge")));

        // Test when mockforge script exists but not watch
        let instructions = npm_integration.generate_setup_instructions(true, false, &templates);
        assert!(instructions
            .iter()
            .any(|s| s.contains("mockforge:watch script")));
        assert!(!instructions.iter().any(|s| s.contains("Add mockforge script")));

        // Test when both exist
        let instructions = npm_integration.generate_setup_instructions(true, true, &templates);
        assert!(!instructions.iter().any(|s| s.contains("Add mockforge script")));
        assert!(!instructions
            .iter()
            .any(|s| s.contains("Add mockforge:watch script")));
        assert!(instructions.iter().any(|s| s.contains("npm run mockforge"))); // Usage instructions
    }

    #[test]
    fn test_validate_npm_setup() {
        let temp_dir = TempDir::new().unwrap();

        // Test with no package.json
        let npm_integration = NpmIntegration::new(temp_dir.path());
        let is_valid = npm_integration.validate_npm_setup().unwrap();
        assert!(!is_valid);

        // Test with package.json but no scripts
        create_test_package_json(temp_dir.path(), None).unwrap();
        let is_valid = npm_integration.validate_npm_setup().unwrap();
        assert!(!is_valid);

        // Test with partial scripts
        create_test_package_json(temp_dir.path(), Some(r#""mockforge": "cargo run --""#)).unwrap();
        let is_valid = npm_integration.validate_npm_setup().unwrap();
        assert!(!is_valid);

        // Test with complete scripts
        create_test_package_json(
            temp_dir.path(),
            Some(r#""mockforge": "cargo run --", "mockforge:watch": "cargo run -- watch""#),
        )
        .unwrap();
        let is_valid = npm_integration.validate_npm_setup().unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_read_package_json_invalid() {
        let temp_dir = TempDir::new().unwrap();
        create_invalid_package_json(temp_dir.path()).unwrap();

        let npm_integration = NpmIntegration::new(temp_dir.path());
        let result = npm_integration.read_package_json();

        assert!(result.is_err());
        if let Err(PulseError::Configuration {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("Invalid JSON"));
            assert!(suggestion.as_ref().unwrap().contains("Fix JSON syntax"));
        } else {
            panic!("Expected configuration error");
        }
    }

    #[test]
    fn test_read_package_json_missing() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());

        let result = npm_integration.read_package_json();
        assert!(result.is_err());

        if let Err(PulseError::FileSystem {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("Cannot read package.json"));
            assert!(suggestion.as_ref().unwrap().contains("package.json exists"));
        } else {
            panic!("Expected filesystem error");
        }
    }

    #[test]
    fn test_write_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let npm_integration = NpmIntegration::new(temp_dir.path());

        let test_json = serde_json::json!({
            "name": "test-project",
            "version": "1.0.0",
            "scripts": {
                "test": "echo test"
            }
        });

        npm_integration.write_package_json(&test_json).unwrap();

        // Verify file was written correctly
        let content = fs::read_to_string(temp_dir.path().join("package.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "test-project");
        assert_eq!(parsed["scripts"]["test"], "echo test");
    }

    #[test]
    fn test_npm_script_template() {
        let template = NpmScriptTemplate {
            name: "test-script".to_string(),
            command: "echo hello".to_string(),
            description: "Test script".to_string(),
        };

        assert_eq!(template.name, "test-script");
        assert_eq!(template.command, "echo hello");
        assert_eq!(template.description, "Test script");
    }

    #[test]
    fn test_npm_setup_status() {
        let status = NpmSetupStatus {
            package_json_exists: true,
            mockforge_script_exists: false,
            mockforge_watch_script_exists: true,
            mockforge_binary_path: Some("test-binary".to_string()),
            recommended_scripts: vec![],
            setup_instructions: vec!["instruction 1".to_string(), "instruction 2".to_string()],
        };

        assert!(status.package_json_exists);
        assert!(!status.mockforge_script_exists);
        assert!(status.mockforge_watch_script_exists);
        assert_eq!(status.mockforge_binary_path.as_ref().unwrap(), "test-binary");
        assert_eq!(status.setup_instructions.len(), 2);
    }
}
