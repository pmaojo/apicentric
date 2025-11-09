//! Sets up or tests the necessary NPM scripts for Apicentric.
//!
//! This module provides a `setup_npm_scripts` function that can be used to
//! configure a project with the necessary NPM scripts for running `apicentric`
//! commands.

use std::path::Path;

use crate::adapters::npm::NpmIntegration;
use crate::errors::ApicentricResult;

/// Sets up or tests the necessary NPM scripts for Apicentric.
///
/// # Arguments
///
/// * `project_root` - The root directory of the project.
/// * `force` - Whether to overwrite existing scripts.
/// * `instructions_only` - Whether to only print the instructions for setting
///   up the scripts manually.
/// * `test` - Whether to test the NPM script execution.
/// * `examples` - Whether to show usage examples for the NPM scripts.
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
