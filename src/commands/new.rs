use apicentric::simulator::marketplace::get_marketplace_items;
use apicentric::simulator::openapi::from_openapi;
use apicentric::simulator::ServiceDefinition;
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

    let template = items.iter().find(|i| i.id == selected_id);

    let template = match template {
        Some(t) => t,
        None => {
            return Err(ApicentricError::Validation {
                message: format!("Template '{}' not found", selected_id),
                field: Some("template".to_string()),
                suggestion: Some(
                    "Check the list of available templates with 'apicentric new --help'"
                        .to_string(),
                ),
            });
        }
    };

    println!("{} Found template: {}", "✅".green(), template.name);
    println!(
        "{} Fetching definition from: {}",
        "⬇️".blue(),
        template.definition_url
    );

    // 3. Download the definition
    let content = reqwest::get(&template.definition_url)
        .await
        .map_err(|e| {
            ApicentricError::network_error(
                format!("Failed to fetch template: {}", e),
                Some(&template.definition_url),
                None::<String>,
            )
        })?
        .text()
        .await
        .map_err(|e| {
            ApicentricError::network_error(
                format!("Failed to read template content: {}", e),
                Some(&template.definition_url),
                None::<String>,
            )
        })?;

    // 4. Parse and update name
    let value: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
        ApicentricError::validation_error(
            format!("Failed to parse template YAML: {}", e),
            None::<String>,
            Some("Check the template syntax"),
        )
    })?;

    let mut definition = if value.get("openapi").is_some() || value.get("swagger").is_some() {
        from_openapi(&value)
    } else {
        serde_yaml::from_value::<ServiceDefinition>(value).map_err(|e| {
            ApicentricError::validation_error(
                format!("Invalid service definition: {}", e),
                None::<String>,
                None::<String>,
            )
        })?
    };

    // Override name with user provided name
    definition.name = name.clone();

    // 5. Save to file
    let services_dir =
        std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    std::fs::create_dir_all(&services_dir).map_err(ApicentricError::Io)?;

    let file_path = Path::new(&services_dir).join(format!("{}.yaml", name));
    if file_path.exists() {
        println!(
            "{} Service file '{}' already exists. Aborting.",
            "⚠️".yellow(),
            file_path.display()
        );
        return Ok(());
    }

    let yaml = serde_yaml::to_string(&definition).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;

    std::fs::write(&file_path, yaml).map_err(ApicentricError::Io)?;

    println!(
        "{} Service created successfully at {}",
        "✨".green(),
        file_path.display()
    );
    println!("Run it with: apicentric simulator start");

    Ok(())
}
