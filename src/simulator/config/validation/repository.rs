use super::super::ServiceDefinition;
use crate::errors::{ApicentricError, ApicentricResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Port for obtaining service definition data
pub trait ConfigRepository {
    fn list_service_files(&self) -> ApicentricResult<Vec<PathBuf>>;
    fn load_service(&self, path: &Path) -> ApicentricResult<ServiceDefinition>;
    fn save_service(&self, path: &Path, content: &str) -> ApicentricResult<()>;
    fn delete_service(&self, path: &Path) -> ApicentricResult<()>;
    fn service_exists(&self, path: &Path) -> bool;
    fn get_services_dir(&self) -> PathBuf;
    fn resolve_path(&self, filename: &str) -> ApicentricResult<PathBuf>;
}

/// Filesystem based implementation of `ConfigRepository`
#[derive(Clone)]
pub struct ConfigFileLoader {
    root: PathBuf,
}

impl ConfigFileLoader {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn is_yaml(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
            .unwrap_or(false)
    }

    fn collect_yaml_recursive(dir: &Path, acc: &mut Vec<PathBuf>) -> ApicentricResult<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot read directory {}: {}", dir.display(), e),
                Some("Check directory permissions"),
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApicentricError::fs_error(
                    format!("Error reading directory entry in {}: {}", dir.display(), e),
                    None::<String>,
                )
            })?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || matches!(name, "node_modules" | "target" | "dist" | "build")
                    {
                        continue;
                    }
                }
                Self::collect_yaml_recursive(&path, acc)?;
            } else if path.is_file() && Self::is_yaml(&path) {
                acc.push(path);
            }
        }
        Ok(())
    }
}

impl ConfigRepository for ConfigFileLoader {
    fn list_service_files(&self) -> ApicentricResult<Vec<PathBuf>> {
        if !self.root.exists() {
            return Err(ApicentricError::config_error(
                format!("Services directory does not exist: {}", self.root.display()),
                Some("Create the services directory and add YAML service definition files"),
            ));
        }
        let mut files = Vec::new();
        Self::collect_yaml_recursive(&self.root, &mut files)?;
        Ok(files)
    }

    fn load_service(&self, path: &Path) -> ApicentricResult<ServiceDefinition> {
        let content = fs::read_to_string(path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Cannot read service file {}: {}", path.display(), e),
                Some("Check file permissions and ensure the file exists"),
            )
        })?;

        // Use UnifiedConfig to support both standard services and digital twins
        let unified: super::super::UnifiedConfig = serde_yaml::from_str(&content).map_err(|e| {
            ApicentricError::config_error(
                format!("Invalid YAML in service file {}: {}", path.display(), e),
                Some("Check YAML syntax and ensure all required fields are present"),
            )
        })?;

        Ok(ServiceDefinition::from(unified))
    }

    fn save_service(&self, path: &Path, content: &str) -> ApicentricResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to create directory {}: {}", parent.display(), e),
                    Some("Check directory permissions"),
                )
            })?;
        }

        fs::write(path, content).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write service file {}: {}", path.display(), e),
                Some("Check file permissions and disk space"),
            )
        })
    }

    fn delete_service(&self, path: &Path) -> ApicentricResult<()> {
        fs::remove_file(path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to delete service file {}: {}", path.display(), e),
                Some("Check file permissions and ensure the file exists"),
            )
        })
    }

    fn service_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn get_services_dir(&self) -> PathBuf {
        self.root.clone()
    }

    fn resolve_path(&self, filename: &str) -> ApicentricResult<PathBuf> {
        let name = Path::new(filename)
            .file_name()
            .ok_or_else(|| {
                ApicentricError::validation_error(
                    "Invalid path: no filename",
                    Some("filename"),
                    Some("Provide a valid filename"),
                )
            })?
            .to_str()
            .ok_or_else(|| {
                ApicentricError::validation_error(
                    "Invalid filename encoding",
                    Some("filename"),
                    None::<String>,
                )
            })?;

        Ok(self.root.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_valid_service(path: &Path) {
        let content = r#"name: test-service
server:
  base_path: /api
endpoints:
  - method: GET
    path: /health
    responses:
      200:
        content_type: application/json
        body: '{}'
"#;
        fs::write(path, content).unwrap();
    }

    #[test]
    fn list_service_files_returns_yaml() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("service.yaml");
        write_valid_service(&file);
        let loader = ConfigFileLoader::new(dir.path().to_path_buf());
        let files = loader.list_service_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file);
    }

    #[test]
    fn load_service_parses_yaml() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("service.yaml");
        write_valid_service(&file);
        let loader = ConfigFileLoader::new(dir.path().to_path_buf());
        let svc = loader.load_service(&file).unwrap();
        assert_eq!(svc.name, "test-service");
    }

    #[test]
    fn list_service_files_missing_dir() {
        let dir = PathBuf::from("/nonexistent-dir");
        let loader = ConfigFileLoader::new(dir.clone());
        let err = loader.list_service_files().unwrap_err();
        assert!(format!("{}", err).contains("does not exist"));
    }
}
