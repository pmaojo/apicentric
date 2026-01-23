use crate::{ApicentricError, ApicentricResult};
use apicentric::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
};
use apicentric::simulator::marketplace::get_marketplace_items;
use apicentric::simulator::openapi::from_openapi;
use colored::Colorize;
use inquire::{Confirm, Select, Text};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Finds YAML files in the specified directory.
///
/// # Arguments
/// * `dir` - The directory path to search in
/// * `recursive` - Whether to search recursively in subdirectories
///
/// # Returns
/// A vector of paths to YAML files found
pub fn find_yaml_files(dir: &Path, recursive: bool) -> ApicentricResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if recursive {
        find_yaml_files_recursive(dir, &mut files)?;
    } else {
        find_yaml_files_in_dir(dir, &mut files)?;
    }
    Ok(files)
}

/// Validates that a file contains valid YAML syntax.
///
/// # Arguments
/// * `file_path` - The path to the YAML file to validate
pub fn validate_yaml_file(file_path: &Path) -> ApicentricResult<()> {
    let content = std::fs::read_to_string(file_path).map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to read file {}: {}", file_path.display(), e),
            Some("Ensure the file exists and is readable"),
        )
    })?;
    let _value: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
        ApicentricError::validation_error(
            format!("Invalid YAML: {}", e),
            None::<String>,
            Some("Check YAML syntax"),
        )
    })?;
    Ok(())
}

fn find_yaml_files_in_dir(dir: &Path, files: &mut Vec<PathBuf>) -> ApicentricResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to read directory {}: {}", dir.display(), e),
            Some("Ensure the directory exists and is readable"),
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to read directory entry: {}", e),
                None::<String>,
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

fn find_yaml_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> ApicentricResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to read directory {}: {}", dir.display(), e),
            Some("Ensure the directory exists and is readable"),
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to read directory entry: {}", e),
                None::<String>,
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    files.push(path);
                }
            }
        } else if path.is_dir() {
            find_yaml_files_recursive(&path, files)?;
        }
    }
    Ok(())
}

/// Fetches a template by ID and creates a service definition file in the output directory.
///
/// If `name_override` is provided, the service name and filename will use it.
/// Otherwise, the template's name (sanitized) is used.
///
/// Returns the path to the created file.
pub async fn fetch_and_create_service(
    template_id: &str,
    name_override: Option<String>,
    output_dir: &str,
) -> ApicentricResult<PathBuf> {
    let items = get_marketplace_items();
    let template = items.iter().find(|i| i.id == template_id).ok_or_else(|| {
        ApicentricError::validation_error(
            format!("Template '{}' not found", template_id),
            Some("template".to_string()),
            Some("Check 'apicentric new --help' for available templates".to_string()),
        )
    })?;

    println!("{} Found template: {}", "✅".green(), template.name);
    println!(
        "{} Fetching definition from: {}",
        "⬇️".blue(),
        template.definition_url
    );

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

    let service_name = name_override.unwrap_or_else(|| {
        // Sanitize template name to be a valid filename/service name
        template.name.to_lowercase().replace(" ", "-")
    });

    definition.name = service_name.clone();

    std::fs::create_dir_all(output_dir).map_err(ApicentricError::Io)?;
    let mut file_path = Path::new(output_dir).join(format!("{}.yaml", service_name));

    if file_path.exists() {
        println!(
            "{} Service file '{}' already exists.",
            "⚠️".yellow(),
            file_path.display()
        );

        // Ask user what to do
        let options = vec!["Overwrite", "Rename", "Cancel"];
        let choice = Select::new("What would you like to do?", options)
            .with_starting_cursor(0)
            .prompt()
            .map_err(|e| {
                ApicentricError::runtime_error(
                    e.to_string(),
                    Some("Interactive prompt failed. Check terminal compatibility"),
                )
            })?;

        match choice {
            "Overwrite" => {
                // Proceed
            }
            "Rename" => {
                let new_name = Text::new("Enter new service name:")
                    .prompt()
                    .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;
                definition.name = new_name.clone();
                file_path = Path::new(output_dir).join(format!("{}.yaml", new_name));
            }
            _ => {
                return Err(ApicentricError::runtime_error(
                    "Operation cancelled by user".to_string(),
                    None::<String>,
                ));
            }
        }
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

    Ok(file_path)
}

/// Prompt the user to create a new [`ServiceDefinition`]
pub fn scaffold_service_definition() -> ApicentricResult<ServiceDefinition> {
    let name = Text::new("Service name:")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let description = Text::new("Description (optional):")
        .prompt_skippable()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let base_path = Text::new("Base path:")
        .with_default("/api")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let port_str = Text::new("Port:")
        .with_default("9000")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;
    let port: u16 = port_str.parse().unwrap_or(9000);

    let mut endpoints = Vec::new();
    loop {
        let endpoint = scaffold_endpoint_definition()?;
        endpoints.push(endpoint);
        let add_more = Confirm::new("Add another endpoint?")
            .with_default(false)
            .prompt()
            .map_err(|e| ApicentricError::runtime_error(
                e.to_string(),
                Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
            ))?;
        if !add_more {
            break;
        }
    }

    Ok(ServiceDefinition {
        name,
        version: None,
        description: description.filter(|s| !s.is_empty()),
        server: Some(ServerConfig {
            port: Some(port),
            base_path,
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        }),
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Some(endpoints),
        graphql: None,
        behavior: None,
        twin: None,
    })
}

/// Prompt the user to create a new [`EndpointDefinition`]
pub fn scaffold_endpoint_definition() -> ApicentricResult<EndpointDefinition> {
    let methods = vec!["GET", "POST", "PUT", "DELETE"];
    let method = Select::new("HTTP method:", methods)
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?
        .to_string();

    let path = Text::new("Path:")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let description = Text::new("Description (optional):")
        .prompt_skippable()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let status_str = Text::new("Response status code:")
        .with_default("200")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;
    let status: u16 = status_str.parse().unwrap_or(200);

    let content_type = Text::new("Content type:")
        .with_default("application/json")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let body = Text::new("Response body:")
        .with_default("{\"message\":\"ok\"}")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let mut responses = HashMap::new();
    responses.insert(
        status,
        ResponseDefinition {
            condition: None,
            content_type,
            body,
            schema: None,
            script: None,
            headers: None,
            side_effects: None,
        },
    );

    Ok(EndpointDefinition {
        kind: EndpointKind::Http,
        method,
        path,
        header_match: None,
        description: description.filter(|s| !s.is_empty()),
        parameters: None,
        request_body: None,
        responses,
        scenarios: None,
        stream: None,
    })
}
