use std::path::Path;

use crate::adapters::npm::NpmIntegration;
use crate::errors::ApicentricResult;

/// Configura o prueba los scripts de npm necesarios para Apicentric.
pub fn setup_npm_scripts(
    project_root: &Path,
    force: bool,
    instructions_only: bool,
    test: bool,
    examples: bool,
) -> ApicentricResult<()> {
    let npm_integration = NpmIntegration::new(project_root);

    if examples {
        npm_integration.show_usage_examples()
    } else if test {
        println!("🧪 Testing npm script configuration...");
        npm_integration.test_npm_scripts()
    } else if instructions_only {
        npm_integration.print_setup_instructions()
    } else {
        println!("⚙️ Setting up npm scripts for apicentric...");
        npm_integration.setup_scripts(force)
    }
}
