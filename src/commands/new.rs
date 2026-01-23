use apicentric::simulator::marketplace::get_marketplace_items;
use apicentric::{ApicentricError, ApicentricResult};
use colored::Colorize;
use std::path::Path;

/// Handles the `new` command.
pub async fn new_command(name: String, template_id: Option<String>) -> ApicentricResult<()> {
    // 1. Get items
    let items = get_marketplace_items();

    // 2. Resolve template ID
    let selected_id = match template_id {
        Some(id) => id,
        None => {
            println!("{} No template specified.", "ℹ️".blue());

            // Try interactive selection if tui feature is enabled (inquire available)
            // But since we can't easily rely on feature flags inside this function without breaking non-tui builds if we import inquire unconditionally,
            // we will stick to a basic list for now, or check if we can do a simple std::io input.
            // For improved DX in this specific step, listing and asking for input is safe.

            println!("Available templates:");
            for item in &items {
                println!("  - {} : {}", item.id.cyan().bold(), item.description);
            }

            println!("\n{} Please enter the template ID to use:", "❓".yellow());
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(ApicentricError::Io)?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                return Err(ApicentricError::Validation {
                    message: "Template ID is required".to_string(),
                    field: Some("template".to_string()),
                    suggestion: Some("Try 'petstore' or 'stripe'".to_string()),
                });
            }
            trimmed
        }
    };

    println!(
        "{} Creating new service '{}' from template '{}'...",
        "📦".blue(),
        name,
        selected_id
    );

    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    
    apicentric::simulator::marketplace::install_template(
        &selected_id,
        Path::new(&services_dir),
        Some(name),
    ).await?;

    println!("Run it with: apicentric simulator start");

    Ok(())
}
