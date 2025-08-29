use clap::{Parser, Subcommand, ValueEnum};
use pulse::{ExecutionContext, PulseError, PulseResult, ContextBuilder};
use pulse::adapters::{CliUiAdapter, MetricsFacade};
use pulse::adapters::cypress::CypressAdapter;
use pulse::adapters::cypress_test_runner::CypressTestRunner;
use pulse::app::run_all::RunAllTestsService;
use pulse::app::watch_impacted::WatchAndRunImpactedService;
use pulse::domain::entities::{RetryPolicy, TestSpec};
use std::sync::Arc;

#[derive(Parser)]
#[command(author, version, about = "Pulse CLI")]
struct Cli {
    /// Path to the pulse.json config file
    #[arg(short, long, default_value = "pulse.json")]
    config: String,

    /// Execution mode (overrides config)
    #[arg(long, value_enum)]
    mode: Option<CliExecutionMode>,

    /// Enable dry-run mode
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum CliExecutionMode { CI, Development, Debug }

impl From<CliExecutionMode> for pulse::config::ExecutionMode {
    fn from(cli_mode: CliExecutionMode) -> Self {
        match cli_mode {
            CliExecutionMode::CI => pulse::config::ExecutionMode::CI,
            CliExecutionMode::Development => pulse::config::ExecutionMode::Development,
            CliExecutionMode::Debug => pulse::config::ExecutionMode::Debug,
        }
    }
}

#[derive(Subcommand)]
enum SimulatorAction {
    Start { #[arg(short, long, default_value = "mock_services")] services_dir: String, #[arg(long)] force: bool },
    Stop { #[arg(long)] force: bool },
    Status { #[arg(short, long)] detailed: bool },
    Validate { #[arg(short, long, default_value = "mock_services")] path: String, #[arg(short, long)] recursive: bool, #[arg(long)] verbose: bool },
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests
    Run { #[arg(short, long, default_value_t = 4)] workers: usize, #[arg(short, long, default_value_t = 0)] retries: usize },
    /// Watch and run impacted tests
    Watch { #[arg(short, long, default_value_t = 4)] workers: usize, #[arg(short, long, default_value_t = 0)] retries: usize, #[arg(long, default_value_t = 800)] debounce_ms: u64 },
    /// API Simulator operations
    Simulator { #[command(subcommand)] action: SimulatorAction },
    /// Docs helper
    Docs { #[arg(long)] serve: bool, #[arg(short, long, default_value = "docs")] output: String, #[arg(short, long)] watch: bool },
    /// Contract comparison for Qualitas public login (mock vs real)
    ContractPublic {
        /// Real API base URL (e.g., https://prev10-backend.qualitascloud.com/api/v1/public)
        #[arg(short = 'r', long = "real-url")]
        real_url: String,
        /// Output report JSON path
        #[arg(short, long, default_value = "contract-public-report.json")]
        output: String,
        /// Keep simulator running after test
        #[arg(long)]
        keep_alive: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await { eprintln!("Error: {}", e); std::process::exit(1); }
}

async fn run(cli: Cli) -> PulseResult<()> {
    let config_path = std::path::Path::new(&cli.config);
    // Build context via builder (same as pulse-sim)
    let builder = ContextBuilder::new(config_path)?;
    let cfg = builder.config().clone();
    let (change_detector, route_indexer, test_runner, junit_adapter, watcher) = pulse::context::init::build_adapters(&cfg);
    let metrics_manager = pulse::context::init::build_metrics_manager(&cfg, &ExecutionContext::new(&cfg));
    let api_simulator = pulse::context::init::build_api_simulator(&cfg);
    let mut context = builder
        .with_change_detector(change_detector)
        .with_route_indexer(route_indexer)
        .with_test_runner(test_runner)
        .with_junit_adapter(junit_adapter)
        .with_metrics_manager(metrics_manager)
        .with_api_simulator(api_simulator)
        .with_watcher(watcher)
        .build()?;
    let mut exec_ctx = ExecutionContext::new(context.config());
    if let Some(mode) = cli.mode { exec_ctx = exec_ctx.with_mode(mode.into()); }
    if cli.dry_run { exec_ctx = exec_ctx.with_dry_run(true); }
    if cli.verbose { exec_ctx = exec_ctx.with_verbose(true); }
    context = context.with_execution_context(exec_ctx.clone());

    match cli.command {
        Commands::Run { workers, retries } => {
            // Via domain service
            let metrics = MetricsFacade::new(context.metrics_manager().clone());
            let ui = CliUiAdapter;
            // Build domain test runner from CypressAdapter
            let cfg = context.config();
            let inner = Arc::new(CypressAdapter::new(cfg.cypress_config_path.clone(), cfg.base_url.clone()));
            let runner = CypressTestRunner::new(inner);
            let service = RunAllTestsService::new(runner, metrics, ui);
            // Discover specs using glob pattern
            let pattern = &context.config().specs_pattern;
            let mut spec_paths: Vec<String> = Vec::new();
            for entry in glob::glob(pattern)? {
                let path = entry?;
                spec_paths.push(path.display().to_string());
            }
            let specs: Vec<TestSpec> = spec_paths.into_iter().map(|p| TestSpec { path: p }).collect();
            let results = match service.run(specs, workers, RetryPolicy { retries: retries as u8 }) {
                Ok(r) => r,
                Err(e) => return Err(PulseError::runtime_error(format!("Run service failed: {}", e), None::<String>)),
            };
            let total = results.len();
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = total - passed;
            println!("\n📊 Test Results: ✅ {} | ❌ {} | 📈 {}", passed, failed, total);
            if failed > 0 { std::process::exit(1); }
            Ok(())
        }
        Commands::Watch { workers, retries, debounce_ms: _ } => {
            // Use domain watch service (continuous)
            let cfg = context.config();
            let inner = Arc::new(CypressAdapter::new(cfg.cypress_config_path.clone(), cfg.base_url.clone()));
            let domain_runner: Arc<dyn pulse::domain::ports::testing::TestRunnerPort + Send + Sync> = Arc::new(CypressTestRunner::new(inner));
            let service = WatchAndRunImpactedService::new(
                context.watcher.clone(),
                context.change_detector.clone(),
                context.route_indexer.clone(),
                domain_runner,
            );
            let root = context.config().routes_dir.display().to_string();
            match service.run(&root, workers, RetryPolicy { retries: retries as u8 }).await {
                Ok(()) => Ok(()),
                Err(e) => Err(PulseError::runtime_error(format!("Watch service failed: {}", e), None::<String>)),
            }
        }
        Commands::Simulator { action } => match action {
            SimulatorAction::Validate { path, recursive, verbose } => {
                // Validate a directory of YAML specs
                let p = std::path::PathBuf::from(path);
                if !p.is_dir() { println!("⚠️  Provide a directory path for validation"); return Ok(()); }
                let cfg = pulse::simulator::config::SimulatorConfig::new(true, p.clone(), pulse::simulator::config::PortRange { start: 9000, end: 9099 });
                let mgr = pulse::simulator::ApiSimulatorManager::new(cfg);
                mgr.validate_configurations()?;
                println!("✅ Validation OK (dir: {})", p.display());
                Ok(())
            }
            SimulatorAction::Start { services_dir:_, force:_ } => {
                if let Some(sim) = context.api_simulator() { sim.start().await?; println!("✅ Simulator started"); } else { println!("⚪ Simulator disabled"); }
                Ok(())
            }
            SimulatorAction::Stop { force:_ } => { if let Some(sim) = context.api_simulator() { sim.stop().await?; println!("🛑 Simulator stopped"); } Ok(()) }
            SimulatorAction::Status { detailed:_ } => { if let Some(sim) = context.api_simulator() { let st = sim.get_status().await; println!("Status: {} services, active: {}", st.services_count, st.is_active); } else { println!("⚪ Simulator disabled"); } Ok(()) }
        },
        Commands::Docs { serve, output, watch } => {
            // Minimal passthrough to existing helper
            let current_dir;
            let project_root = if config_path.is_absolute() { config_path.parent().unwrap_or_else(|| std::path::Path::new(".")) } else { current_dir = std::env::current_dir().map_err(|e| PulseError::fs_error(format!("Cannot determine current directory: {}", e), None::<String>))?; current_dir.as_path() };
            pulse::generate_docs(project_root, &output, serve, watch, false)
        }
        Commands::ContractPublic { real_url, output, keep_alive, verbose } => {
            run_contract_public(&context, &real_url, &output, keep_alive, verbose).await
        }
    }
}

async fn run_contract_public(
    context: &pulse::Context,
    real_url: &str,
    output: &str,
    keep_alive: bool,
    verbose: bool,
) -> PulseResult<()> {
    use reqwest::Client;
    use serde_json::{json, Value};

    // Ensure simulator is running
    let mut started_here = false;
    if let Some(sim) = context.api_simulator() {
        if !sim.is_active().await {
            sim.start().await?;
            started_here = true;
        }
    }

    let mock_base = "http://localhost:9011/api/v1/public";
    let client = Client::builder().danger_accept_invalid_certs(true).build().map_err(|e| PulseError::runtime_error(format!("HTTP client error: {}", e), None::<String>))?;

    let form = [
        ("username", "qcadmin"),
        ("password", "deilua"),
        ("client_id", "1_4eduj4rjcvwg8s0k8gsg0sw4kso0kcgs4sc4occggs0w800scg"),
    ];

    let mock_resp_text = client
        .post(format!("{}/login", mock_base))
        .form(&form)
        .send()
        .await
        .map_err(|e| PulseError::runtime_error(format!("Mock request failed: {}", e), None::<String>))?
        .text()
        .await
        .unwrap_or("{}".to_string());

    let real_resp_text = client
        .post(format!("{}/login", real_url.trim_end_matches('/')))
        .form(&form)
        .send()
        .await
        .map_err(|e| PulseError::runtime_error(format!("Real request failed: {}", e), None::<String>))?
        .text()
        .await
        .unwrap_or("{}".to_string());

    if verbose {
        println!("Mock: {}", &mock_resp_text.chars().take(200).collect::<String>());
        println!("Real: {}", &real_resp_text.chars().take(200).collect::<String>());
    }

    let mock_json: Value = serde_json::from_str(&mock_resp_text).unwrap_or(json!({}));
    let real_json: Value = serde_json::from_str(&real_resp_text).unwrap_or(json!({}));

    // Compare field paths (scalars)
    fn scalar_paths(v: &Value, prefix: String, out: &mut Vec<String>) {
        match v {
            Value::Object(map) => {
                for (k, vv) in map {
                    let p = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                    scalar_paths(vv, p, out);
                }
            }
            Value::Array(arr) => {
                for (i, vv) in arr.iter().enumerate() {
                    scalar_paths(vv, format!("{}[{}]", prefix, i), out);
                }
            }
            _ => out.push(prefix),
        }
    }

    let mut mock_fields = Vec::new();
    let mut real_fields = Vec::new();
    scalar_paths(&mock_json, String::new(), &mut mock_fields);
    scalar_paths(&real_json, String::new(), &mut real_fields);
    mock_fields.sort(); mock_fields.dedup();
    real_fields.sort(); real_fields.dedup();

    let missing_in_real: Vec<_> = mock_fields.iter().filter(|f| !real_fields.contains(f)).cloned().collect();
    let extra_in_real: Vec<_> = real_fields.iter().filter(|f| !mock_fields.contains(f)).cloned().collect();
    let structure_match = missing_in_real.is_empty() && extra_in_real.is_empty();

    let report = json!({
        "endpoint": "POST /login",
        "mock_response": mock_json,
        "real_response": real_json,
        "fields_missing_in_real": missing_in_real,
        "fields_extra_in_real": extra_in_real,
        "structure_match": structure_match,
    });

    std::fs::write(output, serde_json::to_string_pretty(&report).unwrap_or("{}".into()))
        .map_err(|e| PulseError::fs_error(format!("Failed to write report: {}", e), None::<String>))?;
    println!("📄 Report saved to {}", output);

    // Optionally stop simulator if we started it
    if started_here && !keep_alive {
        if let Some(sim) = context.api_simulator() { let _ = sim.stop().await; }
    }
    Ok(())
}
