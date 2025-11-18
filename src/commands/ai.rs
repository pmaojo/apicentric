use apicentric::ai;
use apicentric::{ApicentricResult, Context, ExecutionContext};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AiAction {
    /// Generate a service definition or endpoints from a prompt
    Generate {
        /// Natural language description of the service or endpoints
        prompt: String,
    },
}

pub async fn ai_command(
    action: &AiAction,
    context: &Context,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    match action {
        AiAction::Generate { prompt } => {
            if exec_ctx.dry_run {
                println!("🏃 Dry run: Would generate service from prompt: {}", prompt);
                return Ok(());
            }

            let yaml = ai::generate_service(context, prompt).await?;

            if let Some(sim) = context.api_simulator() {
                let service_name = sim.apply_service_yaml(&yaml).await?;
                println!(
                    "✅ Generated service '{}' applied to simulator",
                    service_name
                );
            } else {
                println!("{}", yaml);
            }

            Ok(())
        }
    }
}
