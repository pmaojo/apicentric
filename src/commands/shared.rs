use crate::{PulseResult, PulseError};
use std::path::{Path, PathBuf};

pub fn find_yaml_files(dir: &Path, recursive: bool) -> PulseResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if recursive { find_yaml_files_recursive(dir, &mut files)?; } else { find_yaml_files_in_dir(dir, &mut files)?; }
    Ok(files)
}

pub fn validate_yaml_file(file_path: &Path) -> PulseResult<()> {
    let content = std::fs::read_to_string(file_path).map_err(|e| PulseError::fs_error(
        format!("Failed to read file {}: {}", file_path.display(), e),
        Some("Ensure the file exists and is readable")
    ))?;
    let _value: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| PulseError::validation_error(
        format!("Invalid YAML: {}", e), None::<String>, Some("Check YAML syntax")
    ))?;
    Ok(())
}

fn find_yaml_files_in_dir(dir: &Path, files: &mut Vec<PathBuf>) -> PulseResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| PulseError::fs_error(
        format!("Failed to read directory {}: {}", dir.display(), e), Some("Ensure the directory exists and is readable")
    ))?;
    for entry in entries {
        let entry = entry.map_err(|e| PulseError::fs_error(format!("Failed to read directory entry: {}", e), None::<String>))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() { if ext == "yaml" || ext == "yml" { files.push(path); } }
        }
    }
    Ok(())
}

fn find_yaml_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> PulseResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| PulseError::fs_error(
        format!("Failed to read directory {}: {}", dir.display(), e), Some("Ensure the directory exists and is readable")
    ))?;
    for entry in entries {
        let entry = entry.map_err(|e| PulseError::fs_error(format!("Failed to read directory entry: {}", e), None::<String>))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() { if ext == "yaml" || ext == "yml" { files.push(path); } }
        } else if path.is_dir() { find_yaml_files_recursive(&path, files)?; }
    }
    Ok(())
}
