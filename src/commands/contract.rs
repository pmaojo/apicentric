use clap::Subcommand;
use apicentric::{ApicentricResult, ExecutionContext, Context, ApicentricError};
use crate::commands::shared::{find_yaml_files};

#[derive(Subcommand, Debug)]
pub enum ContractAction {
    /// Register a new contract from a service definition
    Register {
        /// Service name
        #[arg(short = 'n', long)]
        service: String,
        /// Path to YAML specification file
        #[arg(short = 's', long)]
        spec: String,
        /// Contract description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },
    /// List all registered contracts
    List {
        /// Show detailed contract information
        #[arg(short, long)]
        detailed: bool,
        /// Filter by service name
        #[arg(long)]
        service: Option<String>,
    },
    /// Validate a contract against real API
    Validate {
        /// Contract ID to validate
        #[arg(short, long)]
        contract_id: String,
        /// Environment to test against (prod, staging, dev)
        #[arg(short, long, default_value = "dev")]
        environment: String,
        /// Compatibility policy (strict, moderate, lenient)
        #[arg(short, long, default_value = "moderate")]
        policy: String,
        /// Generate HTML report
        #[arg(long)]
        html_report: bool,
        /// Send notifications
        #[arg(long)]
        notify: bool,
    },
    /// Validate all contracts
    ValidateAll {
        /// Environment to test against
        #[arg(short, long, default_value = "dev")]
        environment: String,
        /// Compatibility policy
        #[arg(short, long, default_value = "moderate")]
        policy: String,
        /// Continue on first failure
        #[arg(long)]
        fail_fast: bool,
        /// Generate comprehensive report
        #[arg(long)]
        report: bool,
    },
    /// Delete a contract
    Delete {
        /// Contract ID to delete
        #[arg(short, long)]
        contract_id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Show contract details
    Show {
        /// Contract ID to show
        #[arg(short, long)]
        contract_id: String,
        /// Show validation history
        #[arg(long)]
        history: bool,
    },
    /// Import contracts from directory
    Import {
        /// Directory containing YAML specifications
        #[arg(short, long, default_value = "mock_services")]
        directory: String,
        /// Import recursively
        #[arg(short, long)]
        recursive: bool,
        /// Overwrite existing contracts
        #[arg(long)]
        overwrite: bool,
    },
    /// Complete contract testing demo
    Demo {
        /// Contract ID to demonstrate
        #[arg(short, long)]
        contract_id: String,
        /// YAML spec file (alternative to registered contract)
        #[arg(long)]
        spec_file: Option<String>,
        /// Mock API port
        #[arg(long, default_value = "8080")]
        mock_port: u16,
        /// Real API base URL
        #[arg(long)]
        real_api_url: Option<String>,
        /// Test endpoints (comma-separated)
        #[arg(long)]
        test_endpoints: Option<String>,
        /// Compatibility policy
        #[arg(short, long, default_value = "moderate")]
        policy: String,
        /// Start simulator automatically
        #[arg(long)]
        auto_start_mock: bool,
        /// Generate detailed HTML report
        #[arg(long)]
        html_report: bool,
        /// Include simulator
        #[arg(long)]
        with_simulator: bool,
        /// Simulator sample size
        #[arg(long, default_value = "2")]
        simulator_sample: usize,
    },
}

pub async fn contract_command(action: &ContractAction, context: &Context, exec_ctx: &ExecutionContext) -> ApicentricResult<()> {
    use apicentric::infrastructure::{FileSystemContractRepository, YamlServiceSpecLoader, ReqwestHttpClientAdapter};
    use apicentric::domain::ports::ServiceSpecLoader;
    use apicentric::domain::contract::*;
    use apicentric::domain::contract_testing::*;

    if exec_ctx.dry_run { print_dry_run(action); return Ok(()); }

    // repo
    let contracts_dir = std::path::Path::new(".apicentric/contracts");
    let repository = FileSystemContractRepository::new(&contracts_dir).await
        .map_err(|e| ApicentricError::config_error(format!("Failed to init contract repository: {}", e), Some("Ensure .apicentric directory is writable")))?;
    let spec_loader = YamlServiceSpecLoader::new();
    let _http_client = ReqwestHttpClientAdapter::new();

    // mocks auxiliares
    struct MockEventPublisher; #[async_trait::async_trait] impl apicentric::domain::ports::ContractEventPublisher for MockEventPublisher { async fn publish(&self, _event: apicentric::ContractEvent) -> Result<(), apicentric::domain::ports::EventError> { Ok(()) } }
    struct MockIdGen; impl apicentric::domain::ports::ContractIdGenerator for MockIdGen { fn generate_contract_id(&self) -> apicentric::ContractId { apicentric::ContractId::new(uuid::Uuid::new_v4().to_string()).unwrap() } fn generate_scenario_id(&self) -> String { uuid::Uuid::new_v4().to_string() } fn generate_validation_id(&self) -> String { uuid::Uuid::new_v4().to_string() } }

    let manage = ManageContractsUseCase::new(repository, spec_loader, Box::new(MockIdGen), Box::new(MockEventPublisher));

    match action {
        ContractAction::Register { service, spec, description } => do_register(&manage, service, spec, description).await?,
        ContractAction::List { detailed, service } => do_list(&manage, *detailed, service).await?,
        ContractAction::Validate { contract_id, environment, policy, html_report, notify } => do_validate(&manage, contract_id, environment, policy, *html_report, *notify).await?,
        ContractAction::ValidateAll { environment, policy, fail_fast, report } => do_validate_all(&manage, environment, policy, *fail_fast, *report).await?,
        ContractAction::Delete { contract_id, yes } => do_delete(&manage, contract_id, *yes).await?,
        ContractAction::Show { contract_id, history } => do_show(&manage, contract_id, *history).await?,
        ContractAction::Import { directory, recursive, overwrite } => do_import(&manage, directory, *recursive, *overwrite).await?,
        ContractAction::Demo { contract_id, spec_file, mock_port, real_api_url, test_endpoints, policy, auto_start_mock, html_report, with_simulator, simulator_sample } => {
            crate::commands::run_full_demo(&manage, context, exec_ctx, contract_id, spec_file.clone(), *mock_port, real_api_url.clone(), test_endpoints.clone(), policy, *auto_start_mock, *html_report, *with_simulator, *simulator_sample).await?;
        }
    }
    Ok(())
}

fn print_dry_run(action: &ContractAction) { println!("🏃 Dry run: {:?}", action); }

async fn do_register<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(manage: &ManageContractsUseCase<T,S>, service: &str, spec: &str, description: &Option<String>) -> ApicentricResult<()> {
    println!("📝 Registering contract: service={} spec={} desc={:?}", service, spec, description);
    match manage.register_contract(service.to_string(), spec.to_string(), description.clone()).await {
        Ok(c) => { println!("✅ Registered: {} for {}", c.id, c.service_name); Ok(()) },
        Err(e) => Err(ApicentricError::runtime_error(format!("Failed: {}", e), Some("Check spec file exists")))
    }
}

async fn do_list<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(manage: &ManageContractsUseCase<T,S>, detailed: bool, service: &Option<String>) -> ApicentricResult<()> {
    let mut items = manage.list_contracts().await.map_err(|e| ApicentricError::runtime_error(
        format!("List error: {}", e),
        Some("Check that the contracts directory exists and is readable")
    ))?;
    if let Some(svc) = service { items = items.into_iter().filter(|c| &c.service_name == svc).collect(); }
    if items.is_empty() { println!("⚠️ No contracts"); return Ok(()); }
    for c in &items { if detailed { println!("🔸 {} {} {} {}", c.id, c.service_name, c.spec_path, c.created_at.format("%Y-%m-%d")); } else { println!("🔸 {} - {}", c.id, c.service_name); } }
    Ok(())
}

async fn do_validate<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(_manage: &ManageContractsUseCase<T,S>, contract_id: &str, environment: &str, policy: &str, html: bool, notify: bool) -> ApicentricResult<()> {
    println!("🔍 Validate contract={} env={} policy={} html_report={} notify={}", contract_id, environment, policy, html, notify);
    println!("⚠️ Validation logic pending real HTTP checks");
    Ok(())
}

async fn do_validate_all<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(manage: &ManageContractsUseCase<T,S>, environment: &str, policy: &str, fail_fast: bool, report: bool) -> ApicentricResult<()> {
    println!("🔍 Validate ALL env={} policy={} fail_fast={} report={}", environment, policy, fail_fast, report);
    let items = manage.list_contracts().await.map_err(|e| ApicentricError::runtime_error(
        format!("List error: {}", e),
        Some("Check that the contracts directory exists and is readable")
    ))?;
    println!("📋 {} contract(s)", items.len());
    Ok(())
}

async fn do_delete<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(manage: &ManageContractsUseCase<T,S>, contract_id: &str, yes: bool) -> ApicentricResult<()> {
    use apicentric::domain::contract_testing::ContractId;
    let id = ContractId::new(contract_id.to_string()).map_err(|e| ApicentricError::validation_error(
        format!("Invalid contract id: {}", e),
        Some("contract_id"),
        Some("Use a valid UUID format for the contract ID")
    ))?;
    if !yes { println!("🗑️ --yes no especificado (skip confirm interactividad en esta versión)" ); }
    match manage.delete_contract(id).await { Ok(_) => println!("✅ Deleted"), Err(e) => println!("❌ Delete error: {}", e) }
    Ok(())
}

async fn do_show<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(manage: &ManageContractsUseCase<T,S>, contract_id: &str, _history: bool) -> ApicentricResult<()> {
    use apicentric::domain::contract_testing::ContractId;
    let id = ContractId::new(contract_id.to_string()).map_err(|e| ApicentricError::validation_error(
        format!("Invalid id: {}", e),
        Some("contract_id"),
        Some("Use a valid UUID format for the contract ID")
    ))?;
    match manage.get_contract(id).await { Ok(Some(c)) => { println!("📋 {} {} {}", c.id, c.service_name, c.spec_path); }, Ok(None) => println!("⚠️ Not found"), Err(e) => println!("❌ Error: {}", e) }
    Ok(())
}

async fn do_import<T: apicentric::domain::ports::ContractRepository, S: apicentric::domain::ports::ServiceSpecLoader>(manage: &ManageContractsUseCase<T,S>, directory: &str, recursive: bool, _overwrite: bool) -> ApicentricResult<()> {
    let dir_path = std::path::Path::new(directory);
    if !dir_path.exists() { return Err(ApicentricError::fs_error(format!("Directory not found: {}", directory), Some("Ensure directory exists"))); }
    let yaml_files = find_yaml_files(dir_path, recursive)?;
    if yaml_files.is_empty() { println!("⚠️ No YAML files"); return Ok(()); }
    println!("📋 Importing {} file(s)", yaml_files.len());
    for f in &yaml_files { let svc = f.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown"); match manage.register_contract(svc.to_string(), f.to_string_lossy().to_string(), Some("Imported".into())).await { Ok(c) => println!("✅ {} => {}", f.display(), c.id), Err(e) => println!("❌ {} ({})", f.display(), e) } }
    Ok(())
}

// handle_demo eliminado: sustituido por run_full_demo
