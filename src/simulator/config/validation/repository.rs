use super::super::ServiceDefinition;
use crate::errors::{ApicentricError, ApicentricResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Port for obtaining service definition data
pub trait ConfigRepository {
    fn list_service_files(&self) -> ApicentricResult<Vec<PathBuf>>;
    fn load_service(&self, path: &Path) -> ApicentricResult<ServiceDefinition>;

    /// Save content to a file in the repository (safe against path traversal)
    fn save_file(&self, filename: &str, content: &str) -> ApicentricResult<()>;

    /// Delete a file from the repository (safe against path traversal)
    fn delete_file(&self, filename: &str) -> ApicentricResult<()>;

    /// Check if a file exists in the repository (safe against path traversal)
    fn file_exists(&self, filename: &str) -> bool;

    /// Read file content from the repository (safe against path traversal)
    fn read_file(&self, filename: &str) -> ApicentricResult<String>;
}

/// Filesystem based implementation of `ConfigRepository`
#[derive(Clone)]
pub struct FileSystemRepository {
    root: PathBuf,
}

/// Backward compatibility alias
pub type ConfigFileLoader = FileSystemRepository;

impl FileSystemRepository {
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

    /// Resolves a safe path for a file, preventing directory traversal.
    fn resolve_path(&self, filename: &str) -> ApicentricResult<PathBuf> {
        let name = match Path::new(filename).file_name() {
            Some(name) => match name.to_str() {
                Some(s) => s,
                None => {
                    return Err(ApicentricError::validation_error(
                        "Invalid filename encoding",
                        None::<String>,
                        None::<String>,
                    ))
                }
            },
            None => {
                return Err(ApicentricError::validation_error(
                    "Invalid path",
                    None::<String>,
                    None::<String>,
                ))
            }
        };

        Ok(self.root.join(name))
    }
}

impl ConfigRepository for FileSystemRepository {
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

    fn save_file(&self, filename: &str, content: &str) -> ApicentricResult<()> {
        let path = self.resolve_path(filename)?;

        // Ensure directory exists
        if !self.root.exists() {
            fs::create_dir_all(&self.root).map_err(|e| {
                ApicentricError::fs_error(
                    format!(
                        "Failed to create services directory {}: {}",
                        self.root.display(),
                        e
                    ),
                    None::<String>,
                )
            })?;
        }

        fs::write(&path, content).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to write file {}: {}", path.display(), e),
                Some("Check disk space and permissions"),
            )
        })
    }

    fn delete_file(&self, filename: &str) -> ApicentricResult<()> {
        let path = self.resolve_path(filename)?;

        if !path.exists() {
            return Err(ApicentricError::runtime_error(
                format!("File not found: {}", filename),
                None::<String>,
            ));
        }

        fs::remove_file(&path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to delete file {}: {}", path.display(), e),
                Some("Check file permissions"),
            )
        })
    }

    fn file_exists(&self, filename: &str) -> bool {
        match self.resolve_path(filename) {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    fn read_file(&self, filename: &str) -> ApicentricResult<String> {
        let path = self.resolve_path(filename)?;

        fs::read_to_string(&path).map_err(|e| {
            ApicentricError::fs_error(
                format!("Failed to read file {}: {}", path.display(), e),
                Some("Check file permissions and ensure the file exists"),
            )
        })
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
        let loader = FileSystemRepository::new(dir.path().to_path_buf());
        let files = loader.list_service_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file);
    }

    #[test]
    fn load_service_parses_yaml() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("service.yaml");
        write_valid_service(&file);
        let loader = FileSystemRepository::new(dir.path().to_path_buf());
        let svc = loader.load_service(&file).unwrap();
        assert_eq!(svc.name, "test-service");
    }

    #[test]
    fn list_service_files_missing_dir() {
        let dir = PathBuf::from("/nonexistent-dir");
        let loader = FileSystemRepository::new(dir.clone());
        let err = loader.list_service_files().unwrap_err();
        assert!(format!("{}", err).contains("does not exist"));
    }

    #[test]
    fn test_save_and_read_file() {
        let dir = tempdir().unwrap();
        let repo = FileSystemRepository::new(dir.path().to_path_buf());
        let filename = "test.txt";
        let content = "hello world";

        repo.save_file(filename, content).unwrap();
        assert!(repo.file_exists(filename));

        let read_content = repo.read_file(filename).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_delete_file() {
        let dir = tempdir().unwrap();
        let repo = FileSystemRepository::new(dir.path().to_path_buf());
        let filename = "test.txt";

        repo.save_file(filename, "content").unwrap();
        assert!(repo.file_exists(filename));

        repo.delete_file(filename).unwrap();
        assert!(!repo.file_exists(filename));
    }

    #[test]
    fn test_path_traversal_prevention() {
        let dir = tempdir().unwrap();
        let repo = FileSystemRepository::new(dir.path().to_path_buf());

        // Attempt traversal
        let filename = "../../etc/passwd";
        let path = repo.resolve_path(filename).unwrap();

        // Should resolve to root/passwd, not /etc/passwd
        assert_eq!(path, dir.path().join("passwd"));

        // Nested traversal
        let filename = "subdir/../test.yaml";
        let path = repo.resolve_path(filename).unwrap();
        assert_eq!(path, dir.path().join("test.yaml"));
    }
}
