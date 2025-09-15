use super::*;
use mockforge::config::{generate_default_config, save_config};
use mockforge::context::{ContextBuilder, ExecutionContext};
use std::fs;
use tempfile::TempDir;

fn build() -> (mockforge::Context, ExecutionContext) {
    let temp = TempDir::new().unwrap();
    let mut config = generate_default_config();
    config.routes_dir = temp.path().join("routes");
    config.specs_dir = temp.path().join("specs");
    config.index_cache_path = temp.path().join("cache").join("index.json");
    if let Some(ref mut sim) = config.simulator {
        sim.services_dir = temp.path().join("services");
    }
    fs::create_dir_all(&config.routes_dir).unwrap();
    fs::create_dir_all(&config.specs_dir).unwrap();
    fs::create_dir_all(config.index_cache_path.parent().unwrap()).unwrap();
    if let Some(ref sim) = config.simulator {
        fs::create_dir_all(&sim.services_dir).unwrap();
    }
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
