use apicentric::{ApicentricResult, Context, ExecutionContext};
// Re-export types used by submodules
pub use apicentric::cli::args::SimulatorAction;

mod control;
mod dockerize;
mod export;
mod import;
mod inspect;
mod service;

pub async fn simulator_command(
    action: &SimulatorAction,
    context: &Context,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    match action {
        SimulatorAction::Start {
            services_dir,
            force,
            template,
        } => {
            control::handle_start(context, services_dir, *force, template.as_deref(), exec_ctx)
                .await
        }
        SimulatorAction::Stop { force } => control::handle_stop(context, *force, exec_ctx).await,
        SimulatorAction::Status { detailed } => {
            control::handle_status(context, *detailed, exec_ctx).await
        }
        SimulatorAction::Validate {
            file,
            recursive,
            verbose,
        } => inspect::handle_validate(file, *recursive, *verbose, exec_ctx).await,
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
        #[cfg(feature = "tui")]
        SimulatorAction::New { output } => service::handle_new(output, exec_ctx).await,
        SimulatorAction::NewGraphql { name, output } => {
            service::handle_new_graphql(name, output, exec_ctx).await
        }
        #[cfg(feature = "tui")]
        SimulatorAction::Edit { file } => service::handle_edit(file, exec_ctx).await,
        SimulatorAction::Record { output, url } => {
            service::handle_record(context, output, url, exec_ctx).await
        }
        SimulatorAction::Dockerize { file, output } => {
            dockerize::handle_dockerize(file, output, exec_ctx).await
        }
        #[cfg(feature = "contract-testing")]
        SimulatorAction::Test {
            path,
            url,
            env,
            quiet,
        } => inspect::handle_contract_test(path, url, env, *quiet, exec_ctx).await,
    }
}
#[cfg(test)]
mod tests;
