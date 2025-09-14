use anyhow::{Result, anyhow};
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;

pub struct SetupNpmHandler;

impl SetupNpmHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, force: bool, instructions_only: bool, test: bool, examples: bool) -> Result<()> {
        if instructions_only {
            self.show_instructions();
            return Ok(());
        }

        if test {
            return self.test_npm_integration();
        }

        if examples {
            self.show_examples();
            return Ok(());
        }

        self.setup_npm_scripts(force)
    }

    fn setup_npm_scripts(&self, force: bool) -> Result<()> {
        println!("📦 Setting up NPM integration scripts...");

        let package_json_path = "package.json";
        if !Path::new(package_json_path).exists() {
            return Err(anyhow!("package.json not found in current directory"));
        }

        // Read package.json
        let content = fs::read_to_string(package_json_path)?;
        let mut package: Value = serde_json::from_str(&content)?;

        // Get or create scripts section
        let scripts = package.get_mut("scripts")
            .and_then(|s| s.as_object_mut())
            .ok_or_else(|| anyhow!("Invalid scripts section in package.json"))?;

        // Define mockforge scripts
        let mockforge_scripts = [
            ("mockforge:build", "cd utils/mockforge && cargo build --release"),
            ("mockforge", "npm run mockforge:build && ./utils/mockforge/target/release/mockforge"),
            ("mockforge:run", "npm run mockforge -- run"),
            ("mockforge:watch", "npm run mockforge -- watch"),
            ("mockforge:debug", "npm run mockforge -- --mode debug --verbose run"),
            ("mockforge:ci", "npm run mockforge -- --mode ci run"),
            ("mockforge:dry", "npm run mockforge -- --dry-run run"),
            ("mockforge:impacted", "npm run mockforge -- watch --dry-run"),
            ("mockforge:report:allure", "allure serve cypress/reports/allure-results"),
            ("mockforge:docs:generate", "npm run mockforge -- docs generate"),
            ("mockforge:docs:serve", "npm run mockforge -- docs serve"),
            ("mockforge:simulator:start", "npm run mockforge -- simulator start"),
            ("mockforge:simulator:stop", "npm run mockforge -- simulator stop"),
            ("mockforge:simulator:status", "npm run mockforge -- simulator status"),
            ("mockforge:contract:demo", "npm run mockforge -- contract demo"),
            ("mockforge:contract:validate", "npm run mockforge -- contract validate"),
        ];

        let mut added = 0;
        let mut skipped = 0;

        for (script_name, script_command) in mockforge_scripts.iter() {
            if scripts.contains_key(*script_name) && !force {
                println!("⏭️  Skipping '{}' (already exists)", script_name);
                skipped += 1;
            } else {
                scripts.insert(script_name.to_string(), Value::String(script_command.to_string()));
                println!("✅ Added script: '{}'", script_name);
                added += 1;
            }
        }

        // Write back to package.json
        let updated_content = serde_json::to_string_pretty(&package)?;
        fs::write(package_json_path, updated_content)?;

        println!("\n📈 Summary:");
        println!("   ✅ Added: {} scripts", added);
        println!("   ⏭️  Skipped: {} scripts", skipped);

        if added > 0 {
            println!("\n🎉 NPM integration setup complete!");
            println!("💡 Try: npm run mockforge:watch");
        }

        Ok(())
    }

    fn show_instructions(&self) {
        println!("📋 NPM Integration Setup Instructions");
        println!("=====================================");
        println!();
        println!("To manually add MockForge scripts to your package.json:");
        println!();
        println!("1. Open your package.json file");
        println!("2. Add these scripts to the 'scripts' section:");
        println!();
        println!(r#"  "scripts": {{
    "mockforge:build": "cd utils/mockforge && cargo build --release",
    "mockforge": "npm run mockforge:build && ./utils/mockforge/target/release/mockforge",
    "mockforge:run": "npm run mockforge -- run",
    "mockforge:watch": "npm run mockforge -- watch",
    "mockforge:debug": "npm run mockforge -- --mode debug --verbose run",
    "mockforge:ci": "npm run mockforge -- --mode ci run",
    "mockforge:dry": "npm run mockforge -- --dry-run run"
  }}"#);
        println!();
        println!("3. Run: npm run mockforge:watch");
        println!();
        println!("💡 Use --force to overwrite existing scripts");
    }

    fn show_examples(&self) {
        println!("📚 MockForge NPM Script Examples");
        println!("============================");
        println!();
        println!("Basic Usage:");
        println!("  npm run mockforge:watch        # Watch mode for development");
        println!("  npm run mockforge:run          # Run all tests once");
        println!("  npm run mockforge:build        # Build Rust binary");
        println!();
        println!("Advanced Usage:");
        println!("  npm run mockforge:debug        # Debug mode with verbose output");
        println!("  npm run mockforge:ci           # CI mode (for pipelines)");
        println!("  npm run mockforge:dry          # Dry run (show what would execute)");
        println!();
        println!("Documentation:");
        println!("  npm run mockforge:docs:generate  # Generate docs");
        println!("  npm run mockforge:docs:serve     # Serve docs locally");
        println!();
        println!("API Simulator:");
        println!("  npm run mockforge:simulator:start   # Start mock API");
        println!("  npm run mockforge:simulator:stop    # Stop mock API");
        println!("  npm run mockforge:simulator:status  # Check status");
        println!();
        println!("Contract Testing:");
        println!("  npm run mockforge:contract:demo      # Run contract demo");
        println!("  npm run mockforge:contract:validate  # Validate contracts");
        println!();
        println!("Reporting:");
        println!("  npm run mockforge:report:allure     # Open Allure reports");
    }

    fn test_npm_integration(&self) -> Result<()> {
        println!("🧪 Testing NPM integration...");

        let package_json_path = "package.json";
        if !Path::new(package_json_path).exists() {
            println!("❌ package.json not found");
            return Err(anyhow!("package.json not found"));
        }

        // Check if scripts exist
        let content = fs::read_to_string(package_json_path)?;
        let package: Value = serde_json::from_str(&content)?;

        let scripts = package.get("scripts")
            .and_then(|s| s.as_object())
            .ok_or_else(|| anyhow!("No scripts section found"))?;

        let required_scripts = ["mockforge:build", "mockforge", "mockforge:run", "mockforge:watch"];
        let mut found = 0;

        for script in required_scripts.iter() {
            if scripts.contains_key(*script) {
                println!("✅ Found script: {}", script);
                found += 1;
            } else {
                println!("❌ Missing script: {}", script);
            }
        }

        if found == required_scripts.len() {
            println!("🎉 All required scripts found!");
            println!("💡 Try running: npm run mockforge:watch");
        } else {
            println!("⚠️  Some scripts are missing. Run: mockforge setup-npm");
        }

        Ok(())
    }
}
