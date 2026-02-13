use super::super::ServiceDefinition;
use crate::errors::{ApicentricError, ApicentricResult};
use crate::utils::fs_utils::FileSystemUtils;
use std::fs;
use std::path::{Path, PathBuf};

/// Port for obtaining service definition data
pub trait ConfigRepository {
    fn list_service_files(&self) -> ApicentricResult<Vec<PathBuf>>;
    fn load_service(&self, path: &Path) -> ApicentricResult<ServiceDefinition>;
    fn save_service_file(&self, filename: &str, content: &str) -> ApicentricResult<()>;
    fn delete_service_file(&self, filename: &str) -> ApicentricResult<()>;
    fn read_service_file(&self, filename: &str) -> ApicentricResult<String>;
    fn service_file_exists(&self, filename: &str) -> bool;
    fn get_services_dir(&self) -> PathBuf;
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

    /// Resolves a filename to a full path within the root directory, ensuring safety.
    fn resolve_path(&self, filename: &str) -> ApicentricResult<PathBuf> {
        let path = self.root.join(filename);

        // Prevent directory traversal
        // Note: We check the string representation for ".." because canonicalize requires existence
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(ApicentricError::validation_error(
                format!("Invalid path '{}': Traversal not allowed", filename),
                Some("filename"),
                Some("Use a path relative to the services directory without '..'"),
            ));
        }

        Ok(path)
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
            // It's okay if the directory doesn't exist, just return empty list
            // But strict behavior requested via validation_error?
            // Existing code returned error. Keeping behavior but creating dir if needed for writes.
            // Actually, list_service_files is read-only.
            // If strict, we return error.
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
        let content = FileSystemUtils::safe_read_file(path)?;

        // Use UnifiedConfig to support both standard services and digital twins
        let unified: super::super::UnifiedConfig = serde_yaml::from_str(&content).map_err(|e| {
            ApicentricError::config_error(
                format!("Invalid YAML in service file {}: {}", path.display(), e),
                Some("Check YAML syntax and ensure all required fields are present"),
            )
        })?;

        Ok(ServiceDefinition::from(unified))
    }

    fn save_service_file(&self, filename: &str, content: &str) -> ApicentricResult<()> {
        let path = self.resolve_path(filename)?;
        FileSystemUtils::safe_write_file(&path, content)
    }

    fn delete_service_file(&self, filename: &str) -> ApicentricResult<()> {
        let path = self.resolve_path(filename)?;
        if !path.exists() {
            return Err(ApicentricError::fs_error(
                format!("Service file '{}' not found", filename),
                Some("Check that the file exists"),
            ));
        }
        fs::remove_file(&path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to delete service file {}: {}", filename, e),
                None::<String>,
            )
        })
    }

    fn read_service_file(&self, filename: &str) -> ApicentricResult<String> {
        let path = self.resolve_path(filename)?;
        FileSystemUtils::safe_read_file(&path)
    }

    fn service_file_exists(&self, filename: &str) -> bool {
        match self.resolve_path(filename) {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    fn get_services_dir(&self) -> PathBuf {
        self.root.clone()
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
