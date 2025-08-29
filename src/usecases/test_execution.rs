use crate::context::Context;
use crate::utils::FileSystemUtils;
use crate::{PulseError, PulseResult};
use std::time::Duration;

pub async fn run_impacted(
    ctx: &Context,
    workers: usize,
    retries: u8,
    headless: bool,
) -> PulseResult<()> {
    let exec = ctx.execution_context();
    if ctx.is_api_simulator_enabled() && !exec.dry_run {
        ctx.start_api_simulator().await?;
    }
    let changed_files = ctx.change_detector.changed_files().map_err(|e| {
        PulseError::fs_error(
            format!("Error getting changed files: {}", e),
            None::<String>,
        )
    })?;
    if changed_files.is_empty() {
        println!("ℹ️ No changes detected, skipping test run");
        return Ok(());
    }
    let specs = ctx
        .route_indexer
        .map_changes_to_specs(&changed_files)
        .map_err(|e| {
            PulseError::fs_error(
                format!("Error mapping changes to specs: {}", e),
                None::<String>,
            )
        })?;
    if specs.is_empty() {
        println!("ℹ️ No tests to run for the changed files");
        return Ok(());
    }
    if exec.dry_run {
        println!("🏃 Dry run impacted ({} specs)", specs.len());
        return Ok(());
    }
    let start = std::time::Instant::now();
    let _ = ctx
        .test_runner
        .run_specs(&specs, workers, retries, headless)?;
    let duration = start.elapsed();
    let report = ctx.junit_adapter.parse_reports()?;
    ctx.junit_adapter.save_consolidated_report(&report)?;
    if let Ok(mut metrics) = ctx.metrics_manager.lock() {
        metrics.record_test_suite_completion(report.total_tests(), report.failed_tests(), duration);
        let _ = metrics.generate_reports();
    }
    if ctx.is_api_simulator_enabled() && !exec.dry_run {
        let _ = ctx.stop_api_simulator().await;
    }
    Ok(())
}

pub async fn run_all(ctx: &Context, workers: usize, retries: u8) -> PulseResult<()> {
    let exec = ctx.execution_context();
    if ctx.is_api_simulator_enabled() && !exec.dry_run {
        ctx.start_api_simulator().await?;
    }
    let spec_paths = FileSystemUtils::resolve_glob_pattern(&ctx.config().specs_pattern, None)?;
    let (valid_specs, _issues) = FileSystemUtils::validate_test_files(&spec_paths);
    if valid_specs.is_empty() {
        return Err(PulseError::fs_error(
            "No valid test files found",
            None::<String>,
        ));
    }
    let specs: Vec<String> = valid_specs
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    if exec.dry_run {
        println!("🏃 Dry run all ({} specs)", specs.len());
        return Ok(());
    }
    let start = std::time::Instant::now();
    let _ = ctx.test_runner.run_specs(&specs, workers, retries, true)?;
    let duration = start.elapsed();
    let report = ctx.junit_adapter.parse_reports()?;
    ctx.junit_adapter.save_consolidated_report(&report)?;
    if let Ok(mut metrics) = ctx.metrics_manager.lock() {
        metrics.record_test_suite_completion(report.total_tests(), report.failed_tests(), duration);
        let _ = metrics.generate_reports();
    }
    if ctx.is_api_simulator_enabled() && !exec.dry_run {
        let _ = ctx.stop_api_simulator().await;
    }
    Ok(())
}

pub async fn watch(
    ctx: &Context,
    workers: usize,
    retries: u8,
    debounce_ms: u64,
) -> PulseResult<()> {
    use crate::domain::ports::testing::WatchEvent;
    let exec = ctx.execution_context();
    if ctx.is_api_simulator_enabled() && !exec.dry_run {
        ctx.start_api_simulator().await?;
    }
    if exec.dry_run {
        println!("🏃 Dry run watch");
        return Ok(());
    }
    let watch_path = ctx.config().routes_dir.to_string_lossy().to_string();
    let mut rx = ctx
        .watcher
        .watch(&watch_path)
        .await
        .map_err(|e| {
            PulseError::runtime_error(format!("Watcher error: {}", e), None::<String>)
        })?;
    println!("🔍 Watching for changes...");
    let debounce = Duration::from_millis(debounce_ms);
    let mut last = std::time::Instant::now();
    while let Some(event) = rx.recv().await {
        if last.elapsed() < debounce {
            continue;
        }
        if matches!(event, WatchEvent::Modified) {
            println!("🔄 Change detected");
            let _ = run_impacted(ctx, workers, retries, true).await;
            last = std::time::Instant::now();
        }
    }
    if ctx.is_api_simulator_enabled() && !exec.dry_run {
        let _ = ctx.stop_api_simulator().await;
    }
    Ok(())
}
