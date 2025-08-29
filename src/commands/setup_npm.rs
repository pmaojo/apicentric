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

        // Define pulse scripts
        let pulse_scripts = [
            ("pulse:build", "cd utils/pulse && cargo build --release"),
            ("pulse", "npm run pulse:build && ./utils/pulse/target/release/pulse"),
            ("pulse:run", "npm run pulse -- run"),
            ("pulse:watch", "npm run pulse -- watch"),
            ("pulse:debug", "npm run pulse -- --mode debug --verbose run"),
            ("pulse:ci", "npm run pulse -- --mode ci run"),
            ("pulse:dry", "npm run pulse -- --dry-run run"),
            ("pulse:impacted", "npm run pulse -- watch --dry-run"),
            ("pulse:report:allure", "allure serve cypress/reports/allure-results"),
            ("pulse:docs:generate", "npm run pulse -- docs generate"),
            ("pulse:docs:serve", "npm run pulse -- docs serve"),
            ("pulse:simulator:start", "npm run pulse -- simulator start"),
            ("pulse:simulator:stop", "npm run pulse -- simulator stop"),
            ("pulse:simulator:status", "npm run pulse -- simulator status"),
            ("pulse:contract:demo", "npm run pulse -- contract demo"),
            ("pulse:contract:validate", "npm run pulse -- contract validate"),
        ];

        let mut added = 0;
        let mut skipped = 0;

        for (script_name, script_command) in pulse_scripts.iter() {
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
            println!("💡 Try: npm run pulse:watch");
        }

        Ok(())
    }

    fn show_instructions(&self) {
        println!("📋 NPM Integration Setup Instructions");
        println!("=====================================");
        println!();
        println!("To manually add Pulse scripts to your package.json:");
        println!();
        println!("1. Open your package.json file");
        println!("2. Add these scripts to the 'scripts' section:");
        println!();
        println!(r#"  "scripts": {{
    "pulse:build": "cd utils/pulse && cargo build --release",
    "pulse": "npm run pulse:build && ./utils/pulse/target/release/pulse",
    "pulse:run": "npm run pulse -- run",
    "pulse:watch": "npm run pulse -- watch",
    "pulse:debug": "npm run pulse -- --mode debug --verbose run",
    "pulse:ci": "npm run pulse -- --mode ci run",
    "pulse:dry": "npm run pulse -- --dry-run run"
  }}"#);
        println!();
        println!("3. Run: npm run pulse:watch");
        println!();
        println!("💡 Use --force to overwrite existing scripts");
    }

    fn show_examples(&self) {
        println!("📚 Pulse NPM Script Examples");
        println!("============================");
        println!();
        println!("Basic Usage:");
        println!("  npm run pulse:watch        # Watch mode for development");
        println!("  npm run pulse:run          # Run all tests once");
        println!("  npm run pulse:build        # Build Rust binary");
        println!();
        println!("Advanced Usage:");
        println!("  npm run pulse:debug        # Debug mode with verbose output");
        println!("  npm run pulse:ci           # CI mode (for pipelines)");
        println!("  npm run pulse:dry          # Dry run (show what would execute)");
        println!();
        println!("Documentation:");
        println!("  npm run pulse:docs:generate  # Generate docs");
        println!("  npm run pulse:docs:serve     # Serve docs locally");
        println!();
        println!("API Simulator:");
        println!("  npm run pulse:simulator:start   # Start mock API");
        println!("  npm run pulse:simulator:stop    # Stop mock API");
        println!("  npm run pulse:simulator:status  # Check status");
        println!();
        println!("Contract Testing:");
        println!("  npm run pulse:contract:demo      # Run contract demo");
        println!("  npm run pulse:contract:validate  # Validate contracts");
        println!();
        println!("Reporting:");
        println!("  npm run pulse:report:allure     # Open Allure reports");
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

        let required_scripts = ["pulse:build", "pulse", "pulse:run", "pulse:watch"];
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
            println!("💡 Try running: npm run pulse:watch");
        } else {
            println!("⚠️  Some scripts are missing. Run: pulse setup-npm");
        }

        Ok(())
    }
}
