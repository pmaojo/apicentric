use super::*;
use mockforge::config::{save_config, PulseConfig};
use mockforge::context::{ContextBuilder, ExecutionContext};
use std::fs;
use tempfile::TempDir;

fn build() -> (mockforge::Context, ExecutionContext) {
    let temp = TempDir::new().unwrap();
    let routes = temp.path().join("routes");
    let specs = temp.path().join("specs");
    let services = temp.path().join("services");
    let cache = temp.path().join("cache");
    fs::create_dir_all(&routes).unwrap();
    fs::create_dir_all(&specs).unwrap();
    fs::create_dir_all(&services).unwrap();
    fs::create_dir_all(&cache).unwrap();
    let config = PulseConfig::builder()
        .routes_dir(routes)
        .specs_dir(specs)
        .index_cache_path(cache.join("index.json"))
        .simulator_services_dir(services)
        .build()
        .unwrap();
    let cfg_path = temp.path().join("mockforge.json");
    save_config(&config, &cfg_path).unwrap();
    let builder = ContextBuilder::new(&cfg_path).unwrap();
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
        &SimulatorAction::Import {
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
async fn export_runs() {
    let (ctx, exec) = build();
    simulator_command(
        &SimulatorAction::Export {
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
