use super::*;
use apicentric::cli::args::ExportFormat;
use apicentric::config::ApicentricConfig;
use apicentric::context::{ContextBuilder, ExecutionContext};
use std::fs;
use tempfile::TempDir;

fn build() -> (apicentric::Context, ExecutionContext) {
    let temp = TempDir::new().unwrap();
    let services = temp.path().join("services");
    fs::create_dir_all(&services).unwrap();

    // Create a minimal config with the new structure
    let config = ApicentricConfig::default();

    let builder = ContextBuilder::new(config);
    let context = builder.build().unwrap();
    let exec = ExecutionContext::new().with_dry_run(true);
    (context, exec)
}

#[tokio::test]
async fn start_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::Start {
            services_dir: "services".into(),
            force: false,
            template: None,
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn new_graphql_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::NewGraphql {
            name: "test-gql".into(),
            output: "services".into(),
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn dockerize_runs() {
    let (ctx, _) = build();
    let exec = ExecutionContext::new().with_dry_run(false);
    let temp = TempDir::new().unwrap();
    let service_path = temp.path().join("service.yaml");
    fs::write(&service_path, "name: test-service\nserver:\n  port: 8080").unwrap();

    simulator_command(
        &SimulatorAction::Dockerize {
            file: vec![service_path.to_str().unwrap().to_string()],
            output: temp.path().to_str().unwrap().to_string(),
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();

    assert!(temp.path().join("Dockerfile").exists());
    assert!(temp.path().join(".dockerignore").exists());
    assert!(temp.path().join("services/service.yaml").exists());
}

#[tokio::test]
async fn validate_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::Validate {
            file: "services".into(),
            recursive: false,
            verbose: false,
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn import_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::Import {
            file: "api.yaml".into(),
            output: "out.yaml".into(),
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn export_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::Export {
            file: "service.yaml".into(),
            output: "openapi.yaml".into(),
            format: ExportFormat::Openapi,
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
#[cfg(feature = "tui")]
async fn new_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::New {
            output: "services".into(),
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}
