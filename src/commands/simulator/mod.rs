use apicentric::{ApicentricResult, Context, ExecutionContext};
<<<<<<< HEAD
use clap::{Subcommand, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum ExportFormat {
    Openapi,
    Postman,
}
=======
use clap::Subcommand;
>>>>>>> origin/main

mod control;
mod dockerize;
mod export;
mod import;
mod inspect;
mod service;

#[derive(Subcommand, Debug)]
pub enum SimulatorAction {
    /// Start the API simulator
    Start {
        /// Services directory path
        #[arg(short, long, default_value = "services")]
        services_dir: String,
        /// Force start even if services are already running
        #[arg(long)]
        force: bool,
        /// Enable peer-to-peer collaboration for service editing
        #[arg(long)]
        p2p: bool,
    },
    /// Stop the API simulator
    Stop {
        /// Force stop all services
        #[arg(long)]
        force: bool,
    },
    /// Show simulator and services status
    Status {
        /// Show detailed service information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Validate service definition files
    Validate {
        /// Path to service definition file or directory
<<<<<<< HEAD
        #[arg(short, long, default_value = "services", alias = "file")]
        file: String,
=======
        #[arg(short, long, default_value = "services")]
        path: String,
>>>>>>> origin/main
        /// Validate all files in directory recursively
        #[arg(short, long)]
        recursive: bool,
        /// Show detailed validation output
        #[arg(long)]
        verbose: bool,
    },
    /// Show recent request logs for a service
    Logs {
        /// Service name
        service: String,
        /// Number of log entries to display
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        /// Filter by HTTP method
        #[arg(long)]
        method: Option<String>,
        /// Filter by route substring
        #[arg(long)]
        route: Option<String>,
        /// Filter by response status code
        #[arg(long)]
        status: Option<u16>,
        /// Output file to write logs as JSON
        #[arg(long)]
        output: Option<String>,
    },
    /// Monitor simulator status and logs
    Monitor {
        /// Service name to monitor
        #[arg(long)]
        service: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Refresh interval in seconds for continuous monitoring
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Set default scenario for all services
    SetScenario {
        /// Scenario name to activate
        scenario: String,
    },
<<<<<<< HEAD
    /// Import a service from a file (OpenAPI, Mockoon, Postman, etc.)
    Import {
        /// Path to the input file to import
        #[arg(short, long, alias = "input")]
        file: String,
        /// Output path for the generated service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Export a service definition to a specified format
    Export {
        /// Path to the service YAML definition
        #[arg(short = 'i', long, alias = "input")]
        file: String,
        /// Output path for the exported file
        #[arg(short, long)]
        output: String,
        /// The format to export to
        #[arg(short, long, value_enum)]
        format: ExportFormat,
    },
    /// Generate TypeScript types for a service
    #[command(name = "generate-types")]
    GenerateTypes {
        /// Path to service YAML definition
        #[arg(short, long, alias = "input")]
        file: String,
=======
    /// Import a service from an OpenAPI spec file
    #[command(name = "import-openapi")]
    ImportOpenapi {
        /// Path to OpenAPI (Swagger) spec
        #[arg(short, long)]
        input: String,
        /// Output path for service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Import a service from a Mockoon export
    ImportMockoon {
        /// Path to Mockoon JSON export
        #[arg(short, long)]
        input: String,
        /// Output path for service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Import a service from WireMock stub mappings
    #[command(name = "import-wiremock")]
    ImportWiremock {
        /// Path to WireMock stub mapping JSON
        #[arg(short, long)]
        input: String,
        /// Output path for service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Import a service from a Postman or Insomnia collection
    #[command(name = "import-postman")]
    ImportPostman {
        /// Path to Postman/Insomnia JSON export
        #[arg(short, long)]
        input: String,
        /// Output path for service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Export a service definition to an OpenAPI spec file
    #[command(name = "export-openapi")]
    ExportOpenapi {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
        /// Output path for OpenAPI spec
        #[arg(short, long)]
        output: String,
    },
    /// Export TypeScript types for a service
    ExportTypes {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
>>>>>>> origin/main
        /// Output path for generated TypeScript
        #[arg(short, long)]
        output: String,
    },
<<<<<<< HEAD
    /// Generate React Query hooks for a service
    #[command(name = "generate-query")]
    GenerateQuery {
        /// Path to service YAML definition
        #[arg(short, long, alias = "input")]
        file: String,
=======
    /// Export React Query hooks for a service
    #[command(name = "export-query")]
    ExportQuery {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
>>>>>>> origin/main
        /// Output path for generated TypeScript hooks
        #[arg(short, long)]
        output: String,
    },
<<<<<<< HEAD
    /// Generate a default React view component for a service
    #[command(name = "generate-view")]
    GenerateView {
        /// Path to service YAML definition
        #[arg(short, long, alias = "input")]
        file: String,
=======
    /// Export a default React view component for a service
    #[command(name = "export-view")]
    ExportView {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
>>>>>>> origin/main
        /// Output path for generated TSX view
        #[arg(short, long)]
        output: String,
    },
<<<<<<< HEAD
=======
    /// Export a service definition to a Postman collection
    #[command(name = "export-postman")]
    ExportPostman {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
        /// Output path for Postman collection JSON
        #[arg(short, long)]
        output: String,
    },
>>>>>>> origin/main
    /// Create a new service definition interactively
    New {
        /// Output directory for the service YAML
        #[arg(short, long, default_value = "services")]
        output: String,
    },
<<<<<<< HEAD
    /// Create a new GraphQL service definition
    #[command(name = "new-graphql")]
    NewGraphql {
        /// The name of the new GraphQL service
        name: String,
        /// Directory to output the new service files
        #[arg(short, long, default_value = "services")]
        output: String,
=======
    /// Create a new GraphQL service definition interactively
    #[command(name = "new-graphql")]
    NewGraphql {
        /// Output directory for the service YAML and GraphQL files
        #[arg(short, long, default_value = "services")]
        output: String,
        /// The name of the service to create (skips interactive prompt)
        #[arg(long)]
        name: Option<String>,
        /// The port for the service (skips interactive prompt)
        #[arg(long)]
        port: Option<u16>,
>>>>>>> origin/main
    },
    /// Edit an existing service definition (add endpoint)
    Edit {
        /// Path to service YAML definition
<<<<<<< HEAD
        #[arg(short, long, alias = "input")]
        file: String,
=======
        #[arg(short, long)]
        input: String,
>>>>>>> origin/main
    },
    /// Record live API interactions into service definitions
    Record {
        /// Output directory for generated YAML services
        #[arg(short, long, default_value = "services")]
        output: String,
        /// Target URL to proxy requests to (defaults to config base_url)
        #[arg(long)]
        url: Option<String>,
    },
    /// Share a running service over libp2p
    Share {
        /// Service name to expose
        service: String,
    },
    /// Connect to a shared service and proxy locally
    Connect {
        /// Remote peer ID
        peer: String,
        /// Service name to access
        #[arg(long)]
        service: String,
        /// Local port to listen on
        #[arg(long)]
        port: u16,
        /// Authentication token issued by the peer
        #[arg(long)]
        token: Option<String>,
    },
    /// Package a service as a Docker image
    Dockerize {
        /// Path to service YAML definition
<<<<<<< HEAD
        #[arg(short, long, alias = "services")]
        file: Vec<String>,
=======
        #[arg(short, long)]
        services: Vec<String>,
>>>>>>> origin/main
        /// Output directory for Docker assets
        #[arg(short, long)]
        output: String,
    },
<<<<<<< HEAD
    /// Run contract tests against a live API
    Test {
        /// Path to the service definition YAML file
        #[arg(short, long)]
        path: String,
        /// The base URL of the live API to test against
        #[arg(short, long)]
        url: String,
        /// The environment name for the test run
        #[arg(long, default_value = "default")]
        env: String,
    },
=======
>>>>>>> origin/main
}

pub async fn simulator_command(
    action: &SimulatorAction,
    context: &Context,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    match action {
        SimulatorAction::Start {
            services_dir,
            force,
            p2p,
        } => control::handle_start(context, services_dir, *force, *p2p, exec_ctx).await,
        SimulatorAction::Stop { force } => control::handle_stop(context, *force, exec_ctx).await,
        SimulatorAction::Status { detailed } => {
            control::handle_status(context, *detailed, exec_ctx).await
        }
        SimulatorAction::Validate {
<<<<<<< HEAD
            file,
            recursive,
            verbose,
        } => inspect::handle_validate(file, *recursive, *verbose, exec_ctx).await,
=======
            path,
            recursive,
            verbose,
        } => inspect::handle_validate(path, *recursive, *verbose, exec_ctx).await,
>>>>>>> origin/main
        SimulatorAction::Logs {
            service,
            limit,
            method,
            route,
            status,
            output,
        } => {
            inspect::handle_logs(
                context,
                service,
                *limit,
                method.as_deref(),
                route.as_deref(),
                *status,
                output.as_deref(),
                exec_ctx,
            )
            .await
        }
        SimulatorAction::Monitor {
            service,
            json,
            interval,
        } => inspect::handle_monitor(context, service.as_deref(), *json, *interval, exec_ctx).await,
        SimulatorAction::SetScenario { scenario } => {
            control::handle_set_scenario(context, scenario, exec_ctx).await
        }
<<<<<<< HEAD
        SimulatorAction::Import { file, output } => {
            import::handle_import(file, output, exec_ctx).await
        }
        SimulatorAction::Export {
            file,
            output,
            format,
        } => export::handle_export(file, output, format, exec_ctx).await,
        SimulatorAction::GenerateTypes { file, output } => {
            export::handle_export_types(file, output, exec_ctx).await
        }
        SimulatorAction::GenerateQuery { file, output } => {
            export::handle_export_query(file, output, exec_ctx).await
        }
        SimulatorAction::GenerateView { file, output } => {
            export::handle_export_view(file, output, exec_ctx).await
        }
        SimulatorAction::New { output } => service::handle_new(output, exec_ctx).await,
        SimulatorAction::NewGraphql { name, output } => {
            service::handle_new_graphql(name, output, exec_ctx).await
        }
        SimulatorAction::Edit { file } => service::handle_edit(file, exec_ctx).await,
=======
        SimulatorAction::ImportOpenapi { input, output } => {
            import::handle_import(input, output, exec_ctx).await
        }
        SimulatorAction::ImportMockoon { input, output } => {
            import::handle_import_mockoon(input, output, exec_ctx).await
        }
        SimulatorAction::ImportWiremock { input, output } => {
            import::handle_import_wiremock(input, output, exec_ctx).await
        }
        SimulatorAction::ImportPostman { input, output } => {
            import::handle_import_postman(input, output, exec_ctx).await
        }
        SimulatorAction::ExportOpenapi { input, output } => {
            export::handle_export(input, output, exec_ctx).await
        }
        SimulatorAction::ExportTypes { input, output } => {
            export::handle_export_types(input, output, exec_ctx).await
        }
        SimulatorAction::ExportQuery { input, output } => {
            export::handle_export_query(input, output, exec_ctx).await
        }
        SimulatorAction::ExportView { input, output } => {
            export::handle_export_view(input, output, exec_ctx).await
        }
        SimulatorAction::ExportPostman { input, output } => {
            export::handle_export_postman(input, output, exec_ctx).await
        }
        SimulatorAction::New { output } => service::handle_new(output, exec_ctx).await,
        SimulatorAction::NewGraphql { output, name, port } => service::handle_new_graphql(output, name, port, exec_ctx).await,
        SimulatorAction::Edit { input } => service::handle_edit(input, exec_ctx).await,
>>>>>>> origin/main
        SimulatorAction::Record { output, url } => {
            service::handle_record(context, output, url, exec_ctx).await
        }
        SimulatorAction::Share { service } => {
            service::handle_share(context, service, exec_ctx).await
        }
        SimulatorAction::Connect {
            peer,
            service,
            port,
            token,
        } => service::handle_connect(peer, service, *port, token.as_deref(), exec_ctx).await,
<<<<<<< HEAD
        SimulatorAction::Dockerize { file, output } => {
            dockerize::handle_dockerize(file, output, exec_ctx).await
        }
        SimulatorAction::Test { path, url, env } => {
            inspect::handle_contract_test(path, url, env, exec_ctx).await
=======
        SimulatorAction::Dockerize { services, output } => {
            dockerize::handle_dockerize(services, output, exec_ctx).await
>>>>>>> origin/main
        }
    }
}
#[cfg(test)]
mod tests;
