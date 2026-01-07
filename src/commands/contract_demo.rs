//! Contract demo (migrated from main.rs)
use apicentric::{ApicentricResult, ApicentricError, Context};
use apicentric::domain::contract_testing::*;

pub async fn run_full_demo<T, S>(
    manage_contracts: &apicentric::domain::contract::ManageContractsUseCase<T, S>,
    context: &Context,
    exec_ctx: &apicentric::ExecutionContext,
    contract_id: &str,
    spec_file: Option<String>,
    mock_port: u16,
    real_api_url: Option<String>,
    test_endpoints: Option<String>,
    policy: &str,
    auto_start_mock: bool,
    html_report: bool,
    with_simulator: bool,
    simulator_sample: usize,
) -> ApicentricResult<()>
where
    T: apicentric::domain::ports::ContractRepository,
    S: apicentric::domain::ports::ServiceSpecLoader,
{
    if exec_ctx.dry_run {
        println!("🏃 Dry run: demo {}", contract_id);
        return Ok(());
    }
    println!("🚀 PULSE CONTRACT TESTING - VALIDACIÓN COMPLETA\n================================================\n");
    let contract = resolve_contract(contract_id, &spec_file, manage_contracts).await?;
    println!("\n🎭 Paso 2: Mock API");
    let mock_base = format!("http://127.0.0.1:{}", mock_port);
    let mock_up = simple_head_or_get(&format!("{}/login", mock_base)).await;

    if !mock_up && auto_start_mock {
        println!("   ▶️ Auto-start mock...");
        if let Err(e) = start_mock_from_contract_spec(&contract, mock_port).await {
            println!("   ❌ Mock start failed: {}", e);
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        }
    }

    let real_base = real_api_url.unwrap_or_else(|| "http://localhost:9010/api/auth".into());
    let endpoints = collect_endpoints(&contract, test_endpoints).await;

    let policy_obj = match policy {
        "strict" => CompatibilityPolicy::strict(),
        "lenient" => CompatibilityPolicy::lenient(),
        _ => CompatibilityPolicy::moderate(),
    };

    let mut all_ok = true;
    let mut rows = Vec::new();

    for ep in endpoints {
        let mock_url = format!("{}{}", mock_base, ep);
        let real_url = format!("{}{}", real_base.trim_end_matches('/'), ep);

        let mock_r = fetch_api_response(&mock_url).await;
        let real_r = fetch_api_response(&real_url).await;

        match (mock_r, real_r) {
            (Ok(m), Ok(r)) => {
                if contract
                    .validate_response_compatibility(&m, &r, &policy_obj)
                    .is_some()
                {
                    all_ok = false;
                }
                rows.push((ep, Some(m), Some(r)));
            }
            (Err(_), Ok(r)) => {
                all_ok = false;
                rows.push((ep, None, Some(r)));
            }
            (Ok(m), Err(_)) => {
                all_ok = false;
                rows.push((ep, Some(m), None));
            }
            (Err(_), Err(_)) => {
                all_ok = false;
                rows.push((ep, None, None));
            }
        }
    }

    if with_simulator {
        if let Some(sim) = context.api_simulator() {
            if !sim.is_active().await {
                let _ = sim.start().await;
            }
            let status = sim.get_status().await;
            println!(
                "🔧 Simulator activo servicios: {}",
                status.active_services.len()
            );
            for svc in status.active_services.iter().take(simulator_sample) {
                println!("   - {}:{}{}", svc.name, svc.port, svc.base_path);
            }
        }
    }

    println!(
        "\nResultado: {}",
        if all_ok {
            "✅ COMPATIBLE"
        } else {
            "❌ DIFERENCIAS"
        }
    );
    if html_report {
        if let Err(e) = write_demo_html_report(&contract, &rows, all_ok) {
            println!("❌ Report error: {}", e);
        }
    }
    Ok(())
}

async fn resolve_contract<T, S>(
    contract_id: &str,
    spec_file: &Option<String>,
    manage: &apicentric::domain::contract::ManageContractsUseCase<T, S>,
) -> ApicentricResult<Contract>
where
    T: apicentric::domain::ports::ContractRepository,
    S: apicentric::domain::ports::ServiceSpecLoader,
{
    if let Some(path) = spec_file {
        use apicentric::domain::ports::ServiceSpecLoader;
        use apicentric::infrastructure::YamlServiceSpecLoader;
        let loader = YamlServiceSpecLoader::new();
        let spec = loader.load(path).await.map_err(|e| {
            ApicentricError::validation_error(
                format!("Spec load error: {}", e),
                None::<String>,
                None::<String>,
            )
        })?;
        return Ok(Contract {
            id: ContractId::new(contract_id.to_string()).map_err(|e| {
                ApicentricError::validation_error(
                    format!("Invalid id: {}", e),
                    None::<String>,
                    None::<String>,
                )
            })?,
            service_name: spec.name.clone(),
            description: Some(format!("Generated from {}", path)),
            spec_path: path.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
    }
    let id = ContractId::new(contract_id.to_string()).map_err(|e| {
        ApicentricError::validation_error(
            format!("Invalid id: {}", e),
            None::<String>,
            None::<String>,
        )
    })?;
    match manage.get_contract(id).await {
        Ok(Some(c)) => Ok(c),
        Ok(None) => Err(ApicentricError::validation_error(
            "Contract not found",
            None::<String>,
            None::<String>,
        )),
        Err(e) => Err(ApicentricError::runtime_error(
            format!("Repo error: {}", e),
            None::<String>,
        )),
    }
}

async fn collect_endpoints(contract: &Contract, explicit: Option<String>) -> Vec<String> {
    if let Some(list) = explicit {
        return list
            .split(',')
            .map(|e| {
                if e.starts_with('/') {
                    e.to_string()
                } else {
                    format!("/{}", e)
                }
            })
            .collect();
    }
    use apicentric::domain::ports::ServiceSpecLoader;
    use apicentric::infrastructure::YamlServiceSpecLoader;
    let loader = YamlServiceSpecLoader::new();
    match loader.load(&contract.spec_path).await {
        Ok(spec) => {
            let v: Vec<String> = spec.endpoints.iter().map(|e| e.path.clone()).collect();
            if v.is_empty() {
                vec!["/login".into(), "/profile".into()]
            } else {
                v
            }
        }
        Err(_) => vec!["/login".into(), "/profile".into()],
    }
}

async fn simple_head_or_get(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    if let Ok(resp) = client.get(url).send().await {
        return resp.status().as_u16() < 600;
    }
    false
}

async fn fetch_api_response(
    url: &str,
) -> Result<apicentric::domain::contract_testing::ApiResponse, String> {
    use apicentric::domain::contract_testing::{ApiResponse, ResponseBody};
    use std::collections::HashMap;
    let start = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect::<HashMap<_, _>>();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    let body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        ResponseBody::Json(json)
    } else {
        ResponseBody::Text(text)
    };
    Ok(ApiResponse::new(
        status,
        headers,
        body,
        start.elapsed().as_millis() as u64,
    ))
}

fn write_demo_html_report(
    contract: &Contract,
    rows: &[(
        String,
        Option<apicentric::domain::contract_testing::ApiResponse>,
        Option<apicentric::domain::contract_testing::ApiResponse>,
    )],
    compatible: bool,
) -> Result<(), String> {
    use chrono::Utc;
    let dir = std::path::Path::new(".apicentric/reports");
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let file = dir.join(format!(
        "demo_contract_{}_{}.html",
        contract.service_name,
        Utc::now().format("%Y%m%d_%H%M%S")
    ));
    let mut html = String::from("<html><head><meta charset='utf-8'><title>Apicentric Contract Demo</title><style>body{font-family:Arial}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:6px;font-size:12px}.ok{color:green}.bad{color:#b00}</style></head><body>");
    html.push_str(&format!(
        "<h1>Contract Demo - {}</h1><p>ID: {}</p><p>Status: <strong class='{}'>{}</strong></p>",
        contract.service_name,
        contract.id,
        if compatible { "ok" } else { "bad" },
        if compatible {
            "COMPATIBLE"
        } else {
            "INCOMPATIBLE"
        }
    ));
    html.push_str("<table><tr><th>Endpoint</th><th>Mock Status</th><th>Real Status</th><th>Match</th></tr>");
    for (ep, m, r) in rows {
        let (ms, rs) = (
            m.as_ref()
                .map(|x| x.status_code.to_string())
                .unwrap_or("ERR".into()),
            r.as_ref()
                .map(|x| x.status_code.to_string())
                .unwrap_or("ERR".into()),
        );
        let match_cell = if let (Some(mv), Some(rv)) = (m, r) {
            if mv.status_code == rv.status_code {
                "✅"
            } else {
                "❌"
            }
        } else {
            "❌"
        };
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            ep, ms, rs, match_cell
        ));
    }
    html.push_str("</table></body></html>");
    std::fs::write(&file, html).map_err(|e| e.to_string())?;
    println!("📄 Reporte HTML: {}", file.display());
    Ok(())
}

async fn start_mock_from_contract_spec(contract: &Contract, port: u16) -> Result<(), String> {
    use apicentric::adapters::mock_server::{MockApiSpec, MockEndpoint};
    use apicentric::domain::ports::ServiceSpecLoader;
    use apicentric::infrastructure::YamlServiceSpecLoader;
    let loader = YamlServiceSpecLoader::new();
    let spec = loader
        .load(&contract.spec_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut mock_spec = MockApiSpec {
        name: Some(spec.name.clone()),
        port: Some(port),
        base_path: Some(spec.base_path.clone()),
        endpoints: Vec::new(),
    };
    for ep in spec.endpoints {
        let body_json = if ep.response.body_template.trim().is_empty() {
            serde_yaml::Value::Null
        } else {
            serde_yaml::from_str(&ep.response.body_template).unwrap_or(serde_yaml::Value::Null)
        };
        mock_spec.endpoints.push(MockEndpoint {
            method: ep.method.to_string(),
            path: ep.path,
            status: ep.response.status,
            delay_ms: None,
            headers: ep.response.headers,
            response: body_json,
        });
    }
    tokio::spawn(async move {
        if let Err(e) = apicentric::adapters::mock_server::run_mock_server(mock_spec).await {
            eprintln!("Mock server error: {}", e);
        }
    });
    Ok(())
}

#[cfg(feature = "tui")]
pub async fn run_tui_contract_demo(
    contract_id: &str,
    spec_file: Option<String>,
    mock_port: u16,
    real_api_url: Option<String>,
    test_endpoints: Option<String>,
    policy: &str,
    auto_start_mock: bool,
) -> Result<Vec<apicentric::tui::app::ContractTestResult>, String> {
    use apicentric::domain::contract_testing::*;
    use apicentric::domain::ports::ServiceSpecLoader;
    use apicentric::infrastructure::YamlServiceSpecLoader;
    use chrono::Local;

    let contract = if let Some(spec_path) = &spec_file {
        let loader = YamlServiceSpecLoader::new();
        let spec = loader
            .load(spec_path)
            .await
            .map_err(|e| format!("Failed to load spec: {}", e))?;
        Contract {
            id: ContractId::new(contract_id.to_string())
                .map_err(|e| format!("Invalid contract ID: {}", e))?,
            service_name: spec.name.clone(),
            description: Some(format!("Generated from spec file: {}", spec_path)),
            spec_path: spec_path.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    } else {
        return Err("No spec file provided".into());
    };

    let endpoints = collect_endpoints(&contract, test_endpoints).await;
    let mock_base = format!("http://127.0.0.1:{}", mock_port);
    let real_base = real_api_url.unwrap_or_else(|| "http://localhost:9010/api/auth".into());

    if auto_start_mock && !simple_head_or_get(&format!("{}/login", mock_base)).await {
        start_mock_from_contract_spec(&contract, mock_port)
            .await
            .map_err(|e| format!("Failed to start mock: {}", e))?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    let mut results = Vec::new();
    for ep in endpoints {
        let mock_url = format!("{}{}", mock_base, ep);
        let real_url = format!("{}{}", real_base.trim_end_matches('/'), ep);

        let mock_resp = fetch_api_response(&mock_url).await;
        let real_resp = fetch_api_response(&real_url).await;

        let result = match (mock_resp, real_resp) {
            (Ok(m), Ok(r)) => {
                let compatible = m.status_code == r.status_code;
                apicentric::tui::app::ContractTestResult {
                    contract_id: contract_id.to_string(),
                    spec_file: spec_file.clone(),
                    endpoint: ep,
                    mock_status: m.status_code,
                    real_status: r.status_code,
                    compatible,
                    error: None,
                    timestamp: Local::now(),
                }
            }
            (Err(me), Ok(r)) => apicentric::tui::app::ContractTestResult {
                contract_id: contract_id.to_string(),
                spec_file: spec_file.clone(),
                endpoint: ep,
                mock_status: 0,
                real_status: r.status_code,
                compatible: false,
                error: Some(format!("Mock error: {}", me)),
                timestamp: Local::now(),
            },
            (Ok(m), Err(re)) => apicentric::tui::app::ContractTestResult {
                contract_id: contract_id.to_string(),
                spec_file: spec_file.clone(),
                endpoint: ep,
                mock_status: m.status_code,
                real_status: 0,
                compatible: false,
                error: Some(format!("Real API error: {}", re)),
                timestamp: Local::now(),
            },
            (Err(me), Err(re)) => apicentric::tui::app::ContractTestResult {
                contract_id: contract_id.to_string(),
                spec_file: spec_file.clone(),
                endpoint: ep,
                mock_status: 0,
                real_status: 0,
                compatible: false,
                error: Some(format!("Both failed - Mock: {}, Real: {}", me, re)),
                timestamp: Local::now(),
            },
        };
        results.push(result);
    }
    Ok(results)
}
