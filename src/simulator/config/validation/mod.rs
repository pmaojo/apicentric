mod repository;
mod summarizer;
mod validators;

pub use repository::{ConfigFileLoader, ConfigRepository};
pub use summarizer::{summarize, LoadError, LoadErrorType, ValidationSummary};
pub use validators::{validate_service_schema, validate_unique_name};

use super::ServiceDefinition;
use crate::errors::{ApicentricError, ApicentricResult};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ConfigLoader<R: ConfigRepository + Clone = repository::ConfigFileLoader> {
    repository: R,
}

impl ConfigLoader<repository::ConfigFileLoader> {
    pub fn new(root: PathBuf) -> Self {
        let canonical_root = root.canonicalize().unwrap_or(root);
        Self {
            repository: repository::ConfigFileLoader::new(canonical_root),
        }
    }
}

impl<R: ConfigRepository + Clone> ConfigLoader<R> {
    pub fn with_repository(repository: R) -> Self {
        Self { repository }
    }

    pub fn load_all_services(&self) -> ApicentricResult<Vec<ServiceDefinition>> {
        let result = self.load_all_services_with_summary()?;
        if result.services.is_empty() {
            return Err(ApicentricError::config_error(
                "No valid service definitions found in services directory",
                Some("Add YAML files with service definitions to the services directory"),
            ));
        }
        Ok(result.services)
    }

    pub fn load_all_services_with_summary(&self) -> ApicentricResult<LoadResult> {
        let files = self.repository.list_service_files()?;
        let mut services = Vec::new();
        let mut errors = Vec::new();
        let mut names = HashSet::new();

        for file in files.iter() {
            match self.repository.load_service(file) {
                Ok(service) => {
                    if let Err(e) = validators::validate_unique_name(&service, &mut names) {
                        errors.push(LoadError {
                            file_path: file.clone(),
                            error_type: LoadErrorType::DuplicateName,
                            message: e.to_string(),
                        });
                    } else if let Err(e) = validators::validate_service_schema(&service) {
                        errors.push(LoadError {
                            file_path: file.clone(),
                            error_type: LoadErrorType::Validation,
                            message: e.to_string(),
                        });
                    } else {
                        services.push(service);
                    }
                }
                Err(e) => {
                    errors.push(LoadError {
                        file_path: file.clone(),
                        error_type: LoadErrorType::Parsing,
                        message: e.to_string(),
                    });
                }
            }
        }

        let summary = summarize(files.len(), errors);
        Ok(LoadResult { services, summary })
    }

    /// Saves a service definition file to the repository.
    pub fn save_service_file(&self, filename: &str, content: &str) -> ApicentricResult<PathBuf> {
        self.repository.save_service_file(filename, content)
    }

    /// Deletes a service definition file from the repository.
    pub fn delete_service_file(&self, filename: &str) -> ApicentricResult<()> {
        self.repository.delete_service_file(filename)
    }

    /// Reads a service definition file from the repository.
    pub fn read_service_file(&self, filename: &str) -> ApicentricResult<String> {
        self.repository.read_service_file(filename)
    }

    /// Checks if a service definition file exists in the repository.
    pub fn service_file_exists(&self, filename: &str) -> bool {
        self.repository.service_file_exists(filename)
    }

    /// Resolves a filename to a safe path within the repository.
    pub fn resolve_path(&self, filename: &str) -> ApicentricResult<PathBuf> {
        self.repository.resolve_path(filename)
    }
}

#[derive(Debug, Clone)]
pub struct LoadResult {
    pub services: Vec<ServiceDefinition>,
    pub summary: ValidationSummary,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_valid_service(path: &PathBuf, name: &str) {
        let content = format!(
            "name: {name}\nserver:\n  base_path: /api\nendpoints:\n  - method: GET\n    path: /health\n    responses:\n      200:\n        content_type: application/json\n        body: '{{}}'\n"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn load_all_services_reports_summary() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("a.yaml");
        let file2 = dir.path().join("b.yaml");
        write_valid_service(&file1, "svc1");
        write_valid_service(&file2, "svc1"); // duplicate name

        let loader = ConfigLoader::new(dir.path().to_path_buf());
        let result = loader.load_all_services_with_summary().unwrap();
        assert_eq!(result.summary.valid_count, 1);
        assert_eq!(result.summary.invalid_count, 1);
        assert_eq!(result.summary.total_files, 2);
    }
}
