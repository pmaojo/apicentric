use super::super::ServiceDefinition;
use crate::errors::{ApicentricError, ApicentricResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Port for obtaining service definition data and managing service files
pub trait ConfigRepository {
    fn list_service_files(&self) -> ApicentricResult<Vec<PathBuf>>;
    fn load_service(&self, path: &Path) -> ApicentricResult<ServiceDefinition>;

    fn save_service_file(&self, filename: &str, content: &str) -> ApicentricResult<PathBuf>;
    fn delete_service_file(&self, filename: &str) -> ApicentricResult<()>;
    fn read_service_file(&self, filename: &str) -> ApicentricResult<String>;
    fn service_file_exists(&self, filename: &str) -> bool;
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

    /// Resolves a safe path for a service file, preventing directory traversal.
    /// It forces the file to be within the root directory by taking only the file_name.
    fn resolve_safe_path(&self, requested_path: &str) -> ApicentricResult<PathBuf> {
        let filename = match Path::new(requested_path).file_name() {
            Some(name) => match name.to_str() {
                Some(s) => s,
                None => {
                    return Err(ApicentricError::validation_error(
                        "Invalid filename encoding",
                        Some("filename"),
                        None::<String>,
                    ))
                }
            },
            None => {
                return Err(ApicentricError::validation_error(
                    "Invalid path",
                    Some("filename"),
                    None::<String>,
                ))
            }
        };

        Ok(self.root.join(filename))
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

    fn save_service_file(&self, filename: &str, content: &str) -> ApicentricResult<PathBuf> {
        let path = self.resolve_safe_path(filename)?;

        // Create services directory if it doesn't exist
        if !self.root.exists() {
            fs::create_dir_all(&self.root).map_err(|e| {
                ApicentricError::fs_error(
                    format!("Failed to create services directory: {}", e),
                    Some("Check directory permissions"),
                )
            })?;
        }

        fs::write(&path, content).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write service file: {}", e),
                Some("Check file permissions"),
            )
        })?;

        Ok(path)
    }

    fn delete_service_file(&self, filename: &str) -> ApicentricResult<()> {
        let path = self.resolve_safe_path(filename)?;

        if !path.exists() {
            return Err(ApicentricError::service_not_found(filename));
        }

        fs::remove_file(&path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to delete service file: {}", e),
                Some("Check file permissions"),
            )
        })?;

        Ok(())
    }

    fn read_service_file(&self, filename: &str) -> ApicentricResult<String> {
        let path = self.resolve_safe_path(filename)?;

        fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ApicentricError::service_not_found(filename)
            } else {
                ApicentricError::fs_error(
                    format!("Failed to read service file: {}", e),
                    Some("Check file permissions"),
                )
            }
        })
    }

    fn service_file_exists(&self, filename: &str) -> bool {
        match self.resolve_safe_path(filename) {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    fn resolve_path(&self, filename: &str) -> ApicentricResult<PathBuf> {
        self.resolve_safe_path(filename)
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

    #[test]
    fn test_save_and_read_service_file() {
        let dir = tempdir().unwrap();
        let loader = ConfigFileLoader::new(dir.path().to_path_buf());
        let filename = "new-service.yaml";
        let content = "name: new-service";

        let path = loader.save_service_file(filename, content).unwrap();
        assert_eq!(path, dir.path().join(filename));
        assert!(loader.service_file_exists(filename));

        let read_content = loader.read_service_file(filename).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_delete_service_file() {
        let dir = tempdir().unwrap();
        let loader = ConfigFileLoader::new(dir.path().to_path_buf());
        let filename = "delete-me.yaml";
        loader.save_service_file(filename, "content").unwrap();
        assert!(loader.service_file_exists(filename));

        loader.delete_service_file(filename).unwrap();
        assert!(!loader.service_file_exists(filename));
    }

    #[test]
    fn test_path_traversal_prevention() {
        let dir = tempdir().unwrap();
        let loader = ConfigFileLoader::new(dir.path().to_path_buf());

        // Try to write outside
        let path = loader.save_service_file("../outside.yaml", "content").unwrap();
        // Should be joined to root/outside.yaml, not root/../outside.yaml
        assert_eq!(path, dir.path().join("outside.yaml"));

        // Verify it didn't write to parent (though tempdir usually cleans up, we check logic)
        assert!(loader.service_file_exists("outside.yaml"));
    }
}
