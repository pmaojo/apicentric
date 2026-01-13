use apicentric::simulator::marketplace::get_marketplace_items;
use apicentric::simulator::openapi::from_openapi;
use apicentric::simulator::ServiceDefinition;
use apicentric::{ApicentricError, ApicentricResult};
use colored::Colorize;
use std::path::Path;

/// Handles the `new` command.
pub async fn new_command(name: String, template_id: String) -> ApicentricResult<()> {
    println!("{} Creating new service '{}' from template '{}'...", "📦".blue(), name, template_id);

    // 1. Find the template
    let items = get_marketplace_items();
    let template = items.iter().find(|i| i.id == template_id);

    let template = match template {
        Some(t) => t,
        None => {
            println!("{} Template '{}' not found.", "❌".red(), template_id);
            println!("Available templates:");
            for item in items {
                println!("  - {} ({})", item.id.green(), item.description);
            }
            return Ok(());
        }
    };

    println!("{} Found template: {}", "✅".green(), template.name);
    println!("{} Fetching definition from: {}", "⬇️".blue(), template.definition_url);

    // 2. Download the definition
    let content = reqwest::get(&template.definition_url)
        .await
        .map_err(|e| ApicentricError::network_error(format!("Failed to fetch template: {}", e), Some(&template.definition_url), None::<String>))?
        .text()
        .await
        .map_err(|e| ApicentricError::network_error(format!("Failed to read template content: {}", e), Some(&template.definition_url), None::<String>))?;

    // 3. Parse and update name
    let value: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| ApicentricError::validation_error(format!("Failed to parse template YAML: {}", e), None::<String>, Some("Check the template syntax")))?;

    let mut definition = if value.get("openapi").is_some() || value.get("swagger").is_some() {
        from_openapi(&value)
    } else {
        serde_yaml::from_value::<ServiceDefinition>(value)
            .map_err(|e| ApicentricError::validation_error(format!("Invalid service definition: {}", e), None::<String>, None::<String>))?
    };

    // Override name with user provided name
    definition.name = name.clone();

    // 4. Save to file
    let services_dir = std::env::var("APICENTRIC_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());
    std::fs::create_dir_all(&services_dir).map_err(ApicentricError::Io)?;

    let file_path = Path::new(&services_dir).join(format!("{}.yaml", name));
    if file_path.exists() {
        println!("{} Service file '{}' already exists. Aborting.", "⚠️".yellow(), file_path.display());
        return Ok(());
    }

    let yaml = serde_yaml::to_string(&definition)
        .map_err(|e| ApicentricError::runtime_error(format!("Failed to serialize service: {}", e), None::<String>))?;

    std::fs::write(&file_path, yaml).map_err(ApicentricError::Io)?;

    println!("{} Service created successfully at {}", "✨".green(), file_path.display());
    println!("Run it with: apicentric simulator start");

    Ok(())
}
