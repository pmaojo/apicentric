use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use apicentric::simulator::log::RequestLogEntry;
use apicentric::{Context, ExecutionContext, ApicentricError, ApicentricResult};

use crate::commands::shared::{find_yaml_files, validate_yaml_file};

pub async fn handle_validate(
    path: &str,
    recursive: bool,
    verbose: bool,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would validate service definitions (path={}, recursive={}, verbose={})",
            path, recursive, verbose
        );
        return Ok(());
    }
    println!("🔍 Validating service definitions...\n📁 Path: {}", path);
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        println!("⚠️ Path does not exist: {}", path);
        return Ok(());
    }
    let files = if path_buf.is_file() {
        vec![path_buf]
    } else {
        find_yaml_files(&path_buf, recursive)?
    };
    let mut valid = 0usize;
    for f in &files {
        if verbose {
            println!("🔎 {}", f.display());
        }
        match validate_yaml_file(f) {
            Ok(_) => {
                valid += 1;
                if verbose {
                    println!("   ✅ valid");
                }
            }
            Err(e) => println!("   ❌ {}", e),
        }
    }
    println!(
        "\n📊 Validation Results: total={} valid={} invalid={}",
        files.len(),
        valid,
        files.len() - valid
    );
    if valid == files.len() {
        println!("✅ All files valid");
    }
    Ok(())
}

pub async fn handle_logs(
    context: &Context,
    service: &str,
    limit: usize,
    method: Option<&str>,
    route: Option<&str>,
    status: Option<u16>,
    output: Option<&str>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would fetch logs for service '{}' (limit={}, method={:?}, route={:?}, status={:?}, output={:?})",
            service, limit, method, route, status, output
        );
        return Ok(());
    }
    if let Some(simulator) = context.api_simulator() {
        let sim_status = simulator.get_status().await;
        if let Some(info) = sim_status
            .active_services
            .iter()
            .find(|s| s.name == service)
        {
            let mut url = format!("http://localhost:{}{}", info.port, info.base_path);
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("__apicentric/logs?limit=");
            url.push_str(&limit.to_string());
            if let Some(m) = method {
                url.push_str("&method=");
                url.push_str(m);
            }
            if let Some(r) = route {
                url.push_str("&route=");
                url.push_str(r);
            }
            if let Some(s) = status {
                url.push_str("&status=");
                url.push_str(&s.to_string());
            }
            let resp = reqwest::get(&url).await.map_err(|e| {
                ApicentricError::runtime_error(format!("Failed to fetch logs: {}", e), None::<String>)
            })?;
            if !resp.status().is_success() {
                return Err(ApicentricError::runtime_error(
                    format!("Failed to fetch logs: status {}", resp.status()),
                    None::<String>,
                ));
            }
            let logs: Vec<RequestLogEntry> = resp.json().await.map_err(|e| {
                ApicentricError::runtime_error(format!("Failed to parse logs: {}", e), None::<String>)
            })?;
            if logs.is_empty() {
                println!("No logs available for service '{}'.", service);
            } else {
                if let Some(path) = output {
                    let file = std::fs::File::create(path).map_err(|e| {
                        ApicentricError::runtime_error(
                            format!("Failed to write logs to {}: {}", path, e),
                            None::<String>,
                        )
                    })?;
                    serde_json::to_writer_pretty(file, &logs).map_err(|e| {
                        ApicentricError::runtime_error(
                            format!("Failed to serialize logs: {}", e),
                            None::<String>,
                        )
                    })?;
                    println!("Saved {} log entries to {}", logs.len(), path);
                } else {
                    for entry in logs {
                        println!(
                            "[{}] {} {} -> {}",
                            entry.timestamp.to_rfc3339(),
                            entry.method,
                            entry.path,
                            entry.status
                        );
                    }
                }
            }
            Ok(())
        } else {
            Err(ApicentricError::runtime_error(
                format!("Service '{}' not found", service),
                Some("Check simulator status for available services"),
            ))
        }
    } else {
        Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ))
    }
}

pub async fn handle_monitor(
    context: &Context,
    service: Option<&str>,
    json: bool,
    interval: Option<u64>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would monitor simulator (service={:?}, json={}, interval={:?})",
            service, json, interval
        );
        return Ok(());
    }

    let simulator = if let Some(sim) = context.api_simulator() {
        sim
    } else {
        return Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ));
    };

    let mut last_seen: HashMap<String, DateTime<Utc>> = HashMap::new();

    loop {
        let status = simulator.get_status().await;

        // Determine which services to check for logs
        let services: Vec<_> = if let Some(name) = service {
            status
                .active_services
                .iter()
                .filter(|s| s.name == name)
                .collect()
        } else {
            status.active_services.iter().collect()
        };

        let mut logs_map: HashMap<String, Vec<RequestLogEntry>> = HashMap::new();
        for svc in services {
            let mut url = format!("http://localhost:{}{}", svc.port, svc.base_path);
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("__apicentric/logs?limit=100");
            let resp = reqwest::get(&url).await.map_err(|e| {
                ApicentricError::runtime_error(format!("Failed to fetch logs: {}", e), None::<String>)
            })?;
            if !resp.status().is_success() {
                return Err(ApicentricError::runtime_error(
                    format!("Failed to fetch logs: status {}", resp.status()),
                    None::<String>,
                ));
            }
            let entries: Vec<RequestLogEntry> = resp.json().await.map_err(|e| {
                ApicentricError::runtime_error(format!("Failed to parse logs: {}", e), None::<String>)
            })?;
            let last = last_seen.get(&svc.name).copied();
            let new_entries: Vec<RequestLogEntry> = match last {
                Some(ts) => entries.into_iter().filter(|e| e.timestamp > ts).collect(),
                None => entries,
            };
            if let Some(max_ts) = new_entries.iter().map(|e| e.timestamp).max() {
                last_seen.insert(svc.name.clone(), max_ts);
            }
            if !new_entries.is_empty() {
                logs_map.insert(svc.name.clone(), new_entries);
            }
        }

        if json {
            let output = serde_json::json!({
                "status": status,
                "logs": logs_map,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            println!("📊 API Simulator Status");
            println!(
                "   Status: {}",
                if status.is_active {
                    "🟢 Running"
                } else {
                    "🔴 Stopped"
                }
            );
            println!("   Services: {} total", status.services_count);
            println!("   Active Services: {}", status.active_services.len());
            for svc in &status.active_services {
                println!(" - {} (port {} base {})", svc.name, svc.port, svc.base_path);
                if let Some(logs) = logs_map.get(&svc.name) {
                    for entry in logs {
                        println!(
                            "   [{}] {} {} -> {}",
                            entry.timestamp.to_rfc3339(),
                            entry.method,
                            entry.path,
                            entry.status
                        );
                    }
                }
            }
        }

        match interval {
            Some(secs) if secs > 0 => {
                use tokio::{
                    signal,
                    time::{sleep, Duration},
                };
                tokio::select! {
                    _ = signal::ctrl_c() => break,
                    _ = sleep(Duration::from_secs(secs)) => {}
                }
            }
            _ => break,
        }
    }

    Ok(())
}
