use apicentric::ai;
use apicentric::{ApicentricResult, Context, ExecutionContext};
use apicentric::cli::args::AiAction;

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

            println!("🤖 Generating service from: \"{}\"...", prompt);

            match ai::generate_service(context, prompt).await {
                Ok(yaml) => {
                    if let Some(sim) = context.api_simulator() {
                        match sim.apply_service_yaml(&yaml).await {
                            Ok(service_name) => {
                                println!(
                                    "✅ Generated service '{}' applied to simulator",
                                    service_name
                                );
                            }
                            Err(e) => {
                                println!("❌ Failed to apply generated service: {}", e);
                            }
                        }
                    } else {
                        println!("{}", yaml);
                    }
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("401") || err_msg.contains("Unauthorized") {
                        println!("\n❌ Authentication Failed: Missing or invalid OPENAI_API_KEY");
                        println!("👉 Please set the OPENAI_API_KEY environment variable to use AI features.");
                        println!("   Example: export OPENAI_API_KEY=sk-...");
                    } else {
                        println!("❌ AI Generation Failed: {}", e);
                    }
                }
            }

            Ok(())
        }
    }
}
