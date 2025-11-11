use super::*;
use apicentric::config::{save_config, ApicentricConfig};
use apicentric::context::{ContextBuilder, ExecutionContext};
use std::fs;
use tempfile::TempDir;

fn build() -> (apicentric::Context, ExecutionContext) {
    let temp = TempDir::new().unwrap();
    let routes = temp.path().join("routes");
    let specs = temp.path().join("specs");
    let services = temp.path().join("services");
    let cache = temp.path().join("cache");
    fs::create_dir_all(&routes).unwrap();
    fs::create_dir_all(&specs).unwrap();
    fs::create_dir_all(&services).unwrap();
    fs::create_dir_all(&cache).unwrap();
    let config = ApicentricConfig::builder()
        .routes_dir(routes)
        .specs_dir(specs)
        .index_cache_path(cache.join("index.json"))
        .simulator_services_dir(services)
        .build()
        .unwrap();
    let builder = ContextBuilder::new(config);
    let context = builder.build().unwrap();
    let exec = ExecutionContext::new(context.config()).with_dry_run(true);
    (context, exec)
}

#[tokio::test]
async fn start_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::Start {
            services_dir: "services".into(),
            force: false,
            p2p: false,
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
    let exec = ExecutionContext::new(ctx.config()).with_dry_run(false);
    let temp = TempDir::new().unwrap();
    let service_path = temp.path().join("service.yaml");
    fs::write(
        &service_path,
        "name: test-service\nserver:\n  port: 8080",
    )
    .unwrap();

    simulator_command(
        &SimulatorAction::Dockerize {
            services: vec![service_path.to_str().unwrap().to_string()],
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
            path: "services".into(),
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
        &SimulatorAction::ImportOpenapi {
            input: "api.yaml".into(),
            output: "out.yaml".into(),
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn import_wiremock_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::ImportWiremock {
            input: "mappings.json".into(),
            output: "service.yaml".into(),
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
        &SimulatorAction::ExportOpenapi {
            input: "service.yaml".into(),
            output: "openapi.yaml".into(),
        },
        &ctx,
        &exec,
    )
    .await
    .unwrap();
}

#[tokio::test]
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
