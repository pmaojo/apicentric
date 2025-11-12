use super::{ApicentricConfig, generate_default_config};
use crate::errors::{ApicentricError, ApicentricResult};
use crate::validation::ConfigValidator;
use std::fs;
use std::path::{Path, PathBuf};

pub trait ConfigRepository {
    fn load(&self) -> ApicentricResult<ApicentricConfig>;
    fn save(&self, config: &ApicentricConfig) -> ApicentricResult<()>;
}

pub struct FileConfigRepository {
    path: PathBuf,
}

impl FileConfigRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl ConfigRepository for FileConfigRepository {
    fn load(&self) -> ApicentricResult<ApicentricConfig> {
        let path = &self.path;
        if !path.exists() {
            return Err(ApicentricError::config_error(
                format!("Configuration file not found: {}", path.display()),
                Some(
                    "Create a apicentric.json file or run 'apicentric init' to generate a default configuration",
                ),
            ));
        }
        let content = fs::read_to_string(path).map_err(|e| {
            ApicentricError::config_error(
                format!("Cannot read configuration file: {}", e),
                Some("Check file permissions and ensure the file is readable"),
            )
        })?;
        let mut config: ApicentricConfig = serde_json::from_str(&content).map_err(|e| {
            ApicentricError::config_error(
                format!("Invalid JSON in configuration file: {}", e),
                Some("Check JSON syntax and ensure all required fields are present"),
            )
        })?;
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        resolve_relative_paths(&mut config, base_dir);
        if let Err(validation_errors) = config.validate() {
            let error_message =
                crate::errors::ErrorFormatter::format_validation_errors(&validation_errors);
            return Err(ApicentricError::config_error(
                format!("Configuration validation failed:\n{}", error_message),
                Some("Fix the validation errors listed above"),
            ));
        }
        Ok(config)
    }

    fn save(&self, config: &ApicentricConfig) -> ApicentricResult<()> {
        let path = &self.path;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Cannot create config directory: {}", e),
                    Some("Check permissions for the parent directory"),
                )
            })?;
        }
        let content = serde_json::to_string_pretty(config).map_err(|e| {
            ApicentricError::config_error(
                format!("Cannot serialize configuration: {}", e),
                None::<String>,
            )
        })?;
        fs::write(path, content).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot write configuration file: {}", e),
                Some("Check write permissions for the target directory"),
            )
        })?;
        Ok(())
    }
}

fn resolve_relative_paths(_config: &mut ApicentricConfig, _base_dir: &Path) {
    // No relative paths to resolve in the simplified config
}

pub fn load_config(path: &Path) -> ApicentricResult<ApicentricConfig> {
    // If config file doesn't exist, return default config
    if !path.exists() {
        log::info!("Config file not found at {:?}, using defaults", path);
        return Ok(ApicentricConfig::default());
    }
    FileConfigRepository::new(path).load()
}

pub fn save_config(config: &ApicentricConfig, path: &Path) -> ApicentricResult<()> {
    FileConfigRepository::new(path).save(config)
}

pub fn init_config(path: &Path) -> ApicentricResult<()> {
    if path.exists() {
        return Err(ApicentricError::config_error(
            format!("Configuration file already exists: {}", path.display()),
            Some("Use --force to overwrite existing configuration or choose a different path"),
        ));
    }
    let default_config = generate_default_config();
    save_config(&default_config, path)?;
    println!("✅ Created default configuration at: {}", path.display());
    println!("💡 Edit the configuration file to customize settings for your project");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_nonexistent_config() {
        let result = load_config(Path::new("nonexistent.json"));
        assert!(result.is_err());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("cfg.json");
        let config = generate_default_config();
        save_config(&config, &path).unwrap();
        let loaded = load_config(&path).unwrap();
        assert!(loaded.ai.is_some());
        assert!(loaded.simulator.is_some());
    }
}

