use crate::errors::{PulseError, PulseResult};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NpmScriptTemplate {
    pub name: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct NpmSetupStatus {
    pub package_json_exists: bool,
    pub mockforge_script_exists: bool,
    pub mockforge_watch_script_exists: bool,
    pub mockforge_binary_path: Option<String>,
    pub recommended_scripts: Vec<NpmScriptTemplate>,
    pub setup_instructions: Vec<String>,
}

pub mod reader;
pub mod writer;

pub use reader::NpmReader;
pub use writer::NpmWriter;

/// Trait defining npm integration capabilities
pub trait NpmIntegrator {
    fn detect_setup_status(&self) -> PulseResult<NpmSetupStatus>;
    fn setup_scripts(&self, force: bool) -> PulseResult<()>;
}

/// Concrete npm integration that delegates to reader and writer components
#[derive(Debug, Clone)]
pub struct NpmIntegration {
    project_root: PathBuf,
    package_json_path: PathBuf,
}

impl NpmIntegration {
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            package_json_path: project_root.join("package.json"),
        }
    }

    fn reader(&self) -> NpmReader {
        NpmReader::new(&self.project_root)
    }

    fn writer(&self) -> NpmWriter {
        NpmWriter::new(&self.project_root)
    }

    /// Detect current npm setup status
    pub fn detect_setup_status(&self) -> PulseResult<NpmSetupStatus> {
        self.reader().detect_setup_status()
    }

    /// Automatically setup npm scripts
    pub fn setup_scripts(&self, force: bool) -> PulseResult<()> {
        self.writer().setup_scripts(force)
    }

    /// Read package.json
    pub fn read_package_json(&self) -> PulseResult<Value> {
        self.reader().read_package_json()
    }

    /// Validate npm setup
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

    /// Print instructions for manual setup
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
        println!("   - npm run mockforge -- run        # Run all tests");
        println!("   - npm run mockforge -- watch      # Watch for changes");
        println!("   - npm run mockforge:watch         # Watch for changes (shortcut)");

        if let Some(binary_path) = &status.mockforge_binary_path {
            println!("\n🔧 Binary path detected: {}", binary_path);
        }

        Ok(())
    }

    /// Show usage examples for npm scripts
    pub fn show_usage_examples(&self) -> PulseResult<()> {
        println!("📚 MockForge NPM Integration Usage Examples");
        println!("======================================");
        println!();
        println!("📁 Working Directory: {}", self.project_root.display());
        println!(
            "   NPM scripts automatically run from the project root where package.json is located",
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
        println!("   cargo run --manifest-path utils/mockforge/Cargo.toml -- setup-npm --instructions-only");
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

        println!("   Testing 'npm run mockforge -- --help'");
        let output = Command::new("npm")
            .args(["run", "mockforge", "--", "--help"])
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

        if status.mockforge_watch_script_exists {
            println!("   Testing 'npm run mockforge:watch -- --help'");
            let output = Command::new("npm")
                .args(["run", "mockforge:watch", "--", "--help"])
                .current_dir(&self.project_root)
                .output()
                .map_err(|e| {
                    PulseError::fs_error(
                        format!(
                            "Failed to execute npm run mockforge:watch from {}: {}",
                            self.project_root.display(),
                            e
                        ),
                        Some(
                            "Check that npm is installed and mockforge:watch script is configured correctly",
                        ),
                    )
                })?;
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

    /// Resolve binary path using reader
    pub fn resolve_mockforge_binary_path(&self) -> PulseResult<String> {
        self.reader().resolve_mockforge_binary_path()
    }
}

impl NpmIntegrator for NpmIntegration {
    fn detect_setup_status(&self) -> PulseResult<NpmSetupStatus> {
        self.reader().detect_setup_status()
    }

    fn setup_scripts(&self, force: bool) -> PulseResult<()> {
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
        assert!(scripts.contains_key("mockforge"));
        assert!(scripts.contains_key("mockforge:watch"));
    }
}
