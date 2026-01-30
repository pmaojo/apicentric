use apicentric::{ApicentricError, ApicentricResult, Context, ExecutionContext};

pub async fn handle_start(
    context: &Context,
    services_dir: &str,
    force: bool,
    template: Option<&str>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    // Install template if provided
    if let Some(template_id) = template {
        if exec_ctx.dry_run {
            println!(
                "🏃 Dry run: Would install template '{}' to '{}'",
                template_id, services_dir
            );
        } else {
            apicentric::simulator::marketplace::install_template(
                template_id,
                std::path::Path::new(services_dir),
                None, // Don't override name, use template name
            )
            .await?;
        }
    }

    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would start API simulator (dir={}, force={})",
            services_dir, force
        );
        return Ok(());
    }
    println!(
        "🚀 Starting API Simulator...\n📁 Services directory: {}",
        services_dir
    );
    if let Some(simulator) = context.api_simulator() {
        if force && simulator.is_active().await {
            println!("🔄 Force stopping existing simulator...");
            simulator.stop().await?;
        }
        match simulator.start().await {
            Ok(_) => {
                let status = simulator.get_status().await;
                println!(
                    "✅ API Simulator started ({} services, {} active)",
                    status.services_count,
                    status.active_services.len()
                );
                for svc in &status.active_services {
                    println!(
                        "   - {}: http://localhost:{}{}",
                        svc.name, svc.port, svc.base_path
                    );
                }
                println!("🔄 Simulator running... Press Ctrl+C to stop");
                tokio::signal::ctrl_c().await.ok();
                println!("🛑 Stopping simulator…");
                simulator.stop().await.ok();
            }
            Err(e) => {
                return Err(ApicentricError::runtime_error(
                    format!("Failed to start simulator: {}", e),
                    Some("Check service configurations and port availability"),
                ))
            }
        }
    } else {
        return Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ));
    }
    Ok(())
}

pub async fn handle_stop(
    context: &Context,
    force: bool,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would stop API simulator (force={})", force);
        return Ok(());
    }
    println!("🛑 Stopping API Simulator...");
    if let Some(simulator) = context.api_simulator() {
        if simulator.is_active().await {
            simulator.stop().await?;
            println!("✅ API Simulator stopped");
        } else {
            println!("⚠️ API Simulator not running");
        }
    } else {
        return Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ));
    }
    Ok(())
}

pub async fn handle_status(
    context: &Context,
    detailed: bool,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would show simulator status (detailed={})",
            detailed
        );
        return Ok(());
    }
    println!("📊 API Simulator Status");
    if let Some(simulator) = context.api_simulator() {
        let status = simulator.get_status().await;
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
        if detailed && !status.active_services.is_empty() {
            println!("\n📋 Service Details:");
            for svc in &status.active_services {
                println!(
                    "   - {} (port {} base {}) endpoints:{} running:{}",
                    svc.name,
                    svc.port,
                    svc.base_path,
                    svc.endpoints_count,
                    if svc.is_running { "yes" } else { "no" }
                );
            }
        }
    } else {
        println!(
            "   Status: ⚪ Not configured\n   💡 Enable simulator in apicentric.json to see status"
        );
    }
    Ok(())
}

pub async fn handle_set_scenario(
    context: &Context,
    scenario: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would set scenario '{}'", scenario);
        return Ok(());
    }

    if let Some(simulator) = context.api_simulator() {
        simulator.set_scenario(Some(scenario.to_string())).await?;
        println!("✅ Scenario set to '{}'", scenario);
        Ok(())
    } else {
        Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ))
    }
}
