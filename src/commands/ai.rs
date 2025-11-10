use clap::Subcommand;
use apicentric::ai::{AiProvider, GeminiAiProvider, LocalAiProvider, OpenAiProvider};
use apicentric::config::AiProviderKind;
use apicentric::{Context, ExecutionContext, ApicentricError, ApicentricResult};

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
        AiAction::Generate { prompt } => handle_ai_generate(context, prompt, exec_ctx).await,
    }
}

async fn handle_ai_generate(
    context: &Context,
    prompt: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would generate service from prompt: {}", prompt);
        return Ok(());
    }

    let cfg = context.config();
    let ai_cfg = match &cfg.ai {
        Some(cfg) => cfg,
        None => {
            return Err(ApicentricError::config_error(
                "AI provider not configured",
                Some("Add an 'ai' section to apicentric.json"),
            ))
        }
    };

    // Build provider based on configuration
    let provider: Box<dyn AiProvider> = match ai_cfg.provider {
        AiProviderKind::Local => {
            let path = ai_cfg
                .model_path
                .clone()
                .unwrap_or_else(|| "model.bin".to_string());
            Box::new(LocalAiProvider::new(path))
        }
        AiProviderKind::Openai => {
            let key = ai_cfg.api_key.clone().ok_or_else(|| {
                ApicentricError::config_error(
                    "OpenAI API key missing",
                    Some("Set ai.api_key in apicentric.json"),
                )
            })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
            Box::new(OpenAiProvider::new(key, model))
        }
        AiProviderKind::Gemini => {
            let key = std::env::var("GEMINI_API_KEY").ok().or_else(|| ai_cfg.api_key.clone()).ok_or_else(|| {
                ApicentricError::config_error(
                    "Gemini API key missing",
                    Some("Set GEMINI_API_KEY environment variable or ai.api_key in apicentric.json"),
                )
            })?;
            let model = ai_cfg
                .model
                .clone()
                .unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
            Box::new(GeminiAiProvider::new(key, model))
        }
    };

    let yaml = provider.generate_yaml(prompt).await?;

    if let Some(sim) = context.api_simulator() {
        sim.apply_service_yaml(&yaml).await?;
        println!("✅ Generated service applied to simulator");
    } else {
        println!("{}", yaml);
    }

    Ok(())
}
