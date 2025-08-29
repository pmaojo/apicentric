use clap::Parser;
use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use pulse::{ContextBuilder, ExecutionContext, PulseError, PulseResult};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use pulse::utils::gamification::{GamificationState, load_gamification, save_gamification};

// Domain adapters/services for actions
use pulse::adapters::cypress::CypressAdapter;
use pulse::adapters::cypress_test_runner::CypressTestRunner;
use pulse::adapters::metrics_facade::MetricsFacade;
use pulse::adapters::ui_cli::CliUiAdapter;
use pulse::app::run_all::RunAllTestsService;

#[derive(Parser)]
#[command(author, version, about = "Pulse TUI (minimal dashboard)")]
struct Cli {
    /// Path to the pulse.json config file
    #[arg(short, long, default_value = "pulse.json")]
    config: String,
}

#[tokio::main]
async fn main() -> PulseResult<()> {
    let cli = Cli::parse();

    // Build context
    let config_path = std::path::Path::new(&cli.config);
    let builder = ContextBuilder::new(config_path)?;
    let cfg = builder.config().clone();
    let (change_detector, route_indexer, test_runner, junit_adapter, watcher) =
        pulse::context::init::build_adapters(&cfg);
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

    let exec = ExecutionContext::new(&cfg);
    context = context.with_execution_context(exec);

    // Setup terminal
    enable_raw_mode().map_err(|e| PulseError::config_error(format!("Raw mode error: {}", e), None::<String>))?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| PulseError::config_error(format!("Terminal error: {}", e), None::<String>))?;
    terminal.clear().ok();

    // State
    let mut selected_tab: usize = 0; // 0 Dashboard, 1 Tests, 2 Simulator, 3 Logs (placeholder)
    let tabs = vec!["Dashboard", "Tests", "Simulator", "Logs"]; 
    let mut last_msg: String = String::from("Ready");
    let mut game: GamificationState = load_gamification(".pulse").unwrap_or_default();
    let mut watch_handle: Option<tokio::task::JoinHandle<()>> = None;

    // Event loop
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| {
            let size = f.size();
            draw_ui(f, size, &tabs, selected_tab, &last_msg, Some(&game));
        }).ok();

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout).unwrap_or(false) {
            if let Ok(CEvent::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Tab => { selected_tab = (selected_tab + 1) % tabs.len(); }
                    KeyCode::BackTab => { selected_tab = (selected_tab + tabs.len() - 1) % tabs.len(); }
                    KeyCode::Char('1') => selected_tab = 0,
                    KeyCode::Char('2') => selected_tab = 1,
                    KeyCode::Char('3') => selected_tab = 2,
                    KeyCode::Char('4') => selected_tab = 3,
                    // Actions
                    KeyCode::Char('r') => {
                        // Run all tests via domain service (updates gamification)
                        let msg = run_all_tests(&context, &mut game).await;
                        let _ = save_gamification(".pulse", &game);
                        last_msg = msg;
                    }
                    KeyCode::Char('s') => {
                        // Toggle simulator start/stop
                        let msg = toggle_simulator(&context).await;
                        last_msg = msg;
                    }
                    KeyCode::Char('w') => {
                        // Toggle watch mode (spawn/abort background task)
                        if let Some(handle) = watch_handle.take() {
                            handle.abort();
                            last_msg = "Watch stopped".into();
                        } else {
                            // Prepare ports and runner for watch service
                            let cfg = context.config().clone();
                            let watcher = context.watcher.clone();
                            let change_detector = context.change_detector.clone();
                            let route_indexer = context.route_indexer.clone();
                            let inner = std::sync::Arc::new(CypressAdapter::new(cfg.cypress_config_path.clone(), cfg.base_url.clone()));
                            let runner: std::sync::Arc<dyn pulse::domain::ports::testing::TestRunnerPort + Send + Sync> = std::sync::Arc::new(CypressTestRunner::new(inner));
                            let root = cfg.routes_dir.display().to_string();
                            watch_handle = Some(tokio::spawn(async move {
                                let _ = run_watch_with_ports(watcher, change_detector, route_indexer, runner, root, 4, 0).await;
                            }));
                            last_msg = "Watch started".into();
                        }
                    }
                    KeyCode::Char('v') => {
                        // Validate simulator
                        let msg = validate_simulator(&context);
                        last_msg = msg;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode().ok();
    Ok(())
}

fn draw_ui(
    f: &mut ratatui::Frame<'_>,
    size: Rect,
    tabs: &[&str],
    selected: usize,
    msg: &str,
    game: Option<&GamificationState>,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(0),    // content
            Constraint::Length(3), // footer
        ])
        .split(size);

    // Tabs
    let tab_widget = Tabs::new(tabs.iter().map(|t| (*t).to_string()).collect::<Vec<String>>())
        .select(selected)
        .block(Block::default().borders(Borders::ALL).title("Pulse TUI"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tab_widget, layout[0]);

    // Content placeholder (Dashboard enriched with gamification)
    let content = match selected {
        0 => {
            let mut base = String::from(
                "Dashboard\n\n[r] Run tests  [w] Toggle watch  [s] Toggle simulator  [v] Validate  [q] Quit\n",
            );
            if let Some(g) = game {
                let total = (g.boss_hp as u64 + 0) as u64; // last run failing specs
                let hp_bar = render_hp_bar(g.boss_hp as usize, 10);
                let best = g.best_run_ms.map(|v| v.to_string()).unwrap_or_else(|| "-".into());
                base.push_str(&format!(
                    "\nBoss HP: {}  (fails: {})\nScore: {}  Streak: {}  Last: {} ms  Best: {} ms\n",
                    hp_bar, g.boss_hp, g.score, g.streak_green, g.last_run_ms, best
                ));
            }
            base
        }
        1 => ("Tests\n\n[r] Run all  [w] Toggle watch").to_string(),
        2 => ("Simulator\n\n[s] Start/Stop  [v] Validate").to_string(),
        _ => ("Logs\n\n(Use this area to stream logs in future)").to_string(),
    };
    let content_p = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Content"));
    f.render_widget(content_p, layout[1]);

    // Footer / status
    let footer = Paragraph::new(format!("Status: {}", msg))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(footer, layout[2]);
}

async fn run_all_tests(context: &pulse::Context, game: &mut GamificationState) -> String {
    // Build domain test runner
    let cfg = context.config();
    let inner = std::sync::Arc::new(CypressAdapter::new(
        cfg.cypress_config_path.clone(),
        cfg.base_url.clone(),
    ));
    let runner = CypressTestRunner::new(inner);
    let metrics = MetricsFacade::new(context.metrics_manager().clone());
    let ui = CliUiAdapter;
    let service = RunAllTestsService::new(runner, metrics, ui);

    // Discover specs (glob pattern)
    let pattern = &cfg.specs_pattern;
    let mut spec_paths: Vec<String> = Vec::new();
    match glob::glob(pattern) {
        Ok(paths) => {
            for entry in paths.flatten() {
                spec_paths.push(entry.display().to_string());
            }
        }
        Err(e) => return format!("Glob error: {}", e),
    }
    if spec_paths.is_empty() {
        return "No specs found".into();
    }
    let specs = spec_paths
        .into_iter()
        .map(|p| pulse::domain::entities::TestSpec { path: p })
        .collect::<Vec<_>>();
    let start = Instant::now();
    match service.run(specs, 4, pulse::domain::entities::RetryPolicy { retries: 0 }) {
        Ok(results) => {
            let total = results.len();
            let passed = results.iter().filter(|r| r.passed).count();
            let ms = start.elapsed().as_millis();
            game.update_after_run(total, passed, ms);
            format!("Tests: {} passed / {} total ({} ms)", passed, total, ms)
        }
        Err(e) => format!("Run error: {}", e),
    }
}

fn render_hp_bar(fails: usize, width: usize) -> String {
    // Map failures to a bar: full when many fails, empty when zero
    let clamped = fails.min(width);
    let filled = "#".repeat(clamped);
    let empty = "-".repeat(width.saturating_sub(clamped));
    format!("[{}{}]", filled, empty)
}

async fn toggle_simulator(context: &pulse::Context) -> String {
    if let Some(sim) = context.api_simulator() {
        let running = sim.is_active().await;
        if running {
            match sim.stop().await { Ok(_) => "Simulator stopped".into(), Err(e) => format!("Stop error: {}", e) }
        } else {
            match sim.start().await { Ok(_) => "Simulator started".into(), Err(e) => format!("Start error: {}", e) }
        }
    } else {
        "Simulator disabled in config".into()
    }
}

fn validate_simulator(context: &pulse::Context) -> String {
    if let Some(sim) = context.api_simulator() {
        match sim.validate_configurations() {
            Ok(names) => format!("Validated {} service(s)", names.len()),
            Err(e) => format!("Validate error: {}", e),
        }
    } else {
        "Simulator disabled in config".into()
    }
}

async fn run_watch_service(context: &pulse::Context, workers: usize, retries: u8) -> Result<(), String> {
    use pulse::adapters::cypress::CypressAdapter;
    use pulse::adapters::cypress_test_runner::CypressTestRunner;
    use pulse::app::watch_impacted::WatchAndRunImpactedService;
    use pulse::domain::entities::RetryPolicy;
    use std::sync::Arc;

    let cfg = context.config();
    let inner = Arc::new(CypressAdapter::new(cfg.cypress_config_path.clone(), cfg.base_url.clone()));
    let domain_runner: Arc<dyn pulse::domain::ports::testing::TestRunnerPort + Send + Sync> = Arc::new(CypressTestRunner::new(inner));

    let service = WatchAndRunImpactedService::new(
        context.watcher.clone(),
        context.change_detector.clone(),
        context.route_indexer.clone(),
        domain_runner,
    );
    let root = cfg.routes_dir.display().to_string();
    service
        .run(&root, workers, RetryPolicy { retries })
        .await
        .map_err(|e| e.to_string())
}

async fn run_watch_with_ports(
    watcher: std::sync::Arc<dyn pulse::domain::ports::testing::WatcherPort + Send + Sync>,
    change_detector: std::sync::Arc<dyn pulse::domain::ports::testing::ChangeDetectorPort + Send + Sync>,
    route_indexer: std::sync::Arc<dyn pulse::domain::ports::testing::RouteIndexerPort + Send + Sync>,
    runner: std::sync::Arc<dyn pulse::domain::ports::testing::TestRunnerPort + Send + Sync>,
    root: String,
    workers: usize,
    retries: u8,
) -> Result<(), String> {
    use pulse::app::watch_impacted::WatchAndRunImpactedService;
    use pulse::domain::entities::RetryPolicy;
    let service = WatchAndRunImpactedService::new(watcher, change_detector, route_indexer, runner);
    service
        .run(&root, workers, RetryPolicy { retries })
        .await
        .map_err(|e| e.to_string())
}
