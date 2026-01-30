use crate::{ApicentricError, ApicentricResult};
use apicentric::simulator::config::validation::{ConfigFileLoader, ConfigRepository};
#[cfg(feature = "tui")]
use apicentric::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
};
#[cfg(feature = "tui")]
use inquire::{Confirm, Select, Text};
#[cfg(feature = "tui")]
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

/// Validates that a file contains valid YAML syntax and conforms to the ServiceDefinition schema.
///
/// # Arguments
/// * `file_path` - The path to the YAML file to validate
pub fn validate_yaml_file(file_path: &Path) -> ApicentricResult<()> {
    // We use ConfigFileLoader which implements ConfigRepository.
    // It requires a root directory, but for single file validation,
    // we can use the file's parent.
    let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
    let loader = ConfigFileLoader::new(parent.to_path_buf());

    // This will attempt to read the file and parse it into UnifiedConfig -> ServiceDefinition
    loader.load_service(file_path).map(|_| ())
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

/// Prompt the user to create a new [`ServiceDefinition`]
#[cfg(feature = "tui")]
pub fn scaffold_service_definition() -> ApicentricResult<ServiceDefinition> {
    let name = Text::new("Service name:")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let description = Text::new("Description (optional):")
        .prompt_skippable()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let base_path = Text::new("Base path:")
        .with_default("/api")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let port_str = Text::new("Port:")
        .with_default("9000")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
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
            .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
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
        description: description.filter(|s: &String| !s.is_empty()),
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
        #[cfg(feature = "iot")]
        twin: None,
    })
}

/// Prompt the user to create a new [`EndpointDefinition`]
#[cfg(feature = "tui")]
pub fn scaffold_endpoint_definition() -> ApicentricResult<EndpointDefinition> {
    let methods = vec!["GET", "POST", "PUT", "DELETE"];
    let method = Select::new("HTTP method:", methods)
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?
        .to_string();

    let path = Text::new("Path:")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let description = Text::new("Description (optional):")
        .prompt_skippable()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let status_str = Text::new("Response status code:")
        .with_default("200")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;
    let status: u16 = status_str.parse().unwrap_or(200);

    let content_type = Text::new("Content type:")
        .with_default("application/json")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let body = Text::new("Response body:")
        .with_default("{\"message\":\"ok\"}")
        .prompt()
        .map_err(|e: inquire::InquireError| ApicentricError::runtime_error(
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
        description: description.filter(|s: &String| !s.is_empty()),
        parameters: None,
        request_body: None,
        responses,
        scenarios: None,
        stream: None,
    })
}
