use crate::{ApicentricError, ApicentricResult};
use apicentric::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
};
use inquire::{Confirm, Select, Text};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

<<<<<<< HEAD
/// Finds YAML files in the specified directory.
///
/// # Arguments
/// * `dir` - The directory path to search in
/// * `recursive` - Whether to search recursively in subdirectories
///
/// # Returns
/// A vector of paths to YAML files found
=======
>>>>>>> origin/main
pub fn find_yaml_files(dir: &Path, recursive: bool) -> ApicentricResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if recursive {
        find_yaml_files_recursive(dir, &mut files)?;
    } else {
        find_yaml_files_in_dir(dir, &mut files)?;
    }
    Ok(files)
}

<<<<<<< HEAD
/// Validates that a file contains valid YAML syntax.
///
/// # Arguments
/// * `file_path` - The path to the YAML file to validate
=======
>>>>>>> origin/main
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
        server: ServerConfig {
            port: Some(port),
            base_path,
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        },
        models: None,
        fixtures: None,
        bucket: None,
        endpoints,
        graphql: None,
        behavior: None,
    })
}

<<<<<<< HEAD
=======
/// Prompt the user to create a new GraphQL [`ServiceDefinition`]
pub fn scaffold_graphql_service_definition() -> ApicentricResult<ServiceDefinition> {
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
        .with_default("/")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;

    let port_str = Text::new("Port:")
        .with_default("9001")
        .prompt()
        .map_err(|e| ApicentricError::runtime_error(
            e.to_string(),
            Some("Interactive prompt failed. Try using non-interactive mode or check terminal compatibility")
        ))?;
    let port: u16 = port_str.parse().unwrap_or(9001);

    Ok(ServiceDefinition {
        name,
        version: None,
        description: description.filter(|s| !s.is_empty()),
        server: ServerConfig {
            port: Some(port),
            base_path,
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        },
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Vec::new(),
        graphql: None,
        behavior: None,
    })
}


>>>>>>> origin/main
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
