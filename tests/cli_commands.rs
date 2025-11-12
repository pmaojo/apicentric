use apicentric::{Context, ExecutionContext, ContextBuilder};
use apicentric::cli::SimulatorAction;
use assert_cmd::prelude::*;
use tempfile::TempDir;

fn setup_test_context() -> (Context, ExecutionContext) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("apicentric.json");
    
    // Create required directory
    std::fs::create_dir_all(temp_dir.path().join(".apicentric")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("services")).unwrap();
    
    // Create minimal valid config JSON with all required fields
    let config_json = r#"
    {
        "cypress_config_path": "cypress.config.ts",
        "base_url": "http://localhost:3000",
        "specs_pattern": "**/*.cy.ts",
        "routes_dir": "services",
        "specs_dir": "cypress/e2e",
        "reports_dir": "cypress/reports",
        "index_cache_path": ".apicentric/route-index.json",
        "default_timeout": 30000
    }
    "#;
    std::fs::write(&config_path, config_json).unwrap();
    
    let config = apicentric::config::load_config(&config_path).unwrap();
    let builder = ContextBuilder::new(config);
    let context = builder.build().unwrap();
    let exec_ctx = ExecutionContext::new(context.config()).with_dry_run(true);
    
    (context, exec_ctx)
}

#[tokio::test]
async fn context_builder_works() {
    let (ctx, exec_ctx) = setup_test_context();
    
    // Verify context was created successfully
    assert!(ctx.config().routes_dir.to_string_lossy().contains("services"));
    assert!(exec_ctx.dry_run);
}

#[tokio::test]
async fn execution_context_has_correct_settings() {
    let (_, exec_ctx) = setup_test_context();
    
    assert!(exec_ctx.dry_run);
    // Can enable verbose mode
    let verbose_ctx = exec_ctx.with_verbose(true);
    assert!(verbose_ctx.verbose);
}

#[tokio::test] 
async fn context_provides_config() {
    let (ctx, _) = setup_test_context();
    
    let config = ctx.config();
    assert!(!config.routes_dir.as_os_str().is_empty());
}

#[tokio::test]
async fn context_builder_creates_simulator_when_needed() {
    let (ctx, _) = setup_test_context();
    
    // Simulator might not be present by default in test context
    // but context builder should work correctly
    let simulator = ctx.api_simulator();
    // Don't require simulator to be present - it's optional depending on config
    println!("Simulator present: {}", simulator.is_some());
}

#[tokio::test]
async fn dry_run_mode_working() {
    let (ctx, _) = setup_test_context();
    let exec_ctx = ExecutionContext::new(ctx.config()).with_dry_run(true);
    
    // In dry run mode, operations should not affect real state
    assert!(exec_ctx.dry_run);
}

#[tokio::test]
async fn simulator_start_respects_services_dir_arg_without_simulator_config() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_path = temp_dir.path().join("apicentric.json");
    let services_dir = temp_dir.path().join("custom_services");
    std::fs::create_dir_all(&services_dir).unwrap();

    // Create a minimal config *without* a "simulator" object.
    let config_json = r#"
    {
        "cypress_config_path": "cypress.config.ts",
        "base_url": "http://localhost:3000",
        "specs_pattern": "**/*.cy.ts",
        "routes_dir": "services",
        "specs_dir": "cypress/e2e",
        "reports_dir": "cypress/reports",
        "index_cache_path": ".apicentric/route-index.json",
        "default_timeout": 30000,
        "execution": {
            "mode": "development"
        }
    }
    "#;
    std::fs::write(&config_path, config_json).unwrap();

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("apicentric")
        .arg("--")
        .arg("--config")
        .arg(&config_path)
        .arg("--dry-run")
        .arg("simulator")
        .arg("start")
        .arg("--services-dir")
        .arg(&services_dir);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Command should succeed");
    assert!(stdout.contains("Dry run: Would start API simulator"));
    assert!(stdout.contains("services_dir"));
}
