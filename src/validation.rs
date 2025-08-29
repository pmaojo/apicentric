use crate::errors::ValidationError;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// Trait for validating configuration objects
pub trait ConfigValidator {
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
}

/// Validation utilities for common configuration patterns
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate that a path exists and is accessible
    pub fn validate_path_exists(path: &Path, field_name: &str) -> Result<(), ValidationError> {
        if !path.exists() {
            return Err(ValidationError::new(
                field_name,
                format!("Path does not exist: {}", path.display()),
            )
            .with_suggestion(format!("Create the directory: mkdir -p {}", path.display())));
        }
        Ok(())
    }

    /// Validate that a directory exists. Optionally create it if missing.
    pub fn validate_directory(
        path: &Path,
        field_name: &str,
        create_if_missing: bool,
    ) -> Result<(), ValidationError> {
        if path.exists() {
            if !path.is_dir() {
                return Err(ValidationError::new(
                    field_name,
                    format!("Path exists but is not a directory: {}", path.display()),
                )
                .with_suggestion("Remove the file or choose a different path"));
            }
        } else if create_if_missing {
            // Try to create the directory
            if let Err(e) = fs::create_dir_all(path) {
                return Err(ValidationError::new(
                    field_name,
                    format!("Cannot create directory {}: {}", path.display(), e),
                )
                .with_suggestion("Check permissions and parent directory existence"));
            }
        } else {
            return Err(ValidationError::new(
                field_name,
                format!("Directory does not exist: {}", path.display()),
            )
            .with_suggestion(format!("Create the directory: mkdir -p {}", path.display())));
        }
        Ok(())
    }

    /// Validate that a file exists and is readable
    pub fn validate_file_exists(path: &Path, field_name: &str) -> Result<(), ValidationError> {
        if !path.exists() {
            return Err(ValidationError::new(
                field_name,
                format!("File does not exist: {}", path.display()),
            )
            .with_suggestion("Create the file or check the path"));
        }

        if !path.is_file() {
            return Err(ValidationError::new(
                field_name,
                format!("Path exists but is not a file: {}", path.display()),
            ));
        }

        // Test readability
        if let Err(e) = fs::File::open(path) {
            return Err(ValidationError::new(
                field_name,
                format!("Cannot read file {}: {}", path.display(), e),
            )
            .with_suggestion("Check file permissions"));
        }

        Ok(())
    }

    /// Validate URL format
    pub fn validate_url(url_str: &str, field_name: &str) -> Result<(), ValidationError> {
        match Url::parse(url_str) {
            Ok(url) => {
                if url.scheme() != "http" && url.scheme() != "https" {
                    return Err(ValidationError::new(
                        field_name,
                        format!("URL must use http or https scheme: {}", url_str),
                    )
                    .with_suggestion("Use format: http://localhost:5173 or https://example.com"));
                }
                Ok(())
            }
            Err(e) => Err(
                ValidationError::new(field_name, format!("Invalid URL format: {}", e))
                    .with_suggestion("Use format: http://localhost:5173"),
            ),
        }
    }

    /// Validate glob pattern
    pub fn validate_glob_pattern(pattern: &str, field_name: &str) -> Result<(), ValidationError> {
        match glob::Pattern::new(pattern) {
            Ok(_) => Ok(()),
            Err(e) => Err(
                ValidationError::new(field_name, format!("Invalid glob pattern: {}", e))
                    .with_suggestion("Check glob pattern syntax, e.g., 'app/**/*.cy.ts'"),
            ),
        }
    }

    /// Validate that a numeric value is within a reasonable range
    pub fn validate_numeric_range<T>(
        value: T,
        min: T,
        max: T,
        field_name: &str,
    ) -> Result<(), ValidationError>
    where
        T: PartialOrd + std::fmt::Display + Copy,
    {
        if value < min || value > max {
            return Err(ValidationError::new(
                field_name,
                format!("Value {} is outside valid range [{}, {}]", value, min, max),
            )
            .with_suggestion(format!("Use a value between {} and {}", min, max)));
        }
        Ok(())
    }

    /// Validate that a string is not empty
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::new(field_name, "Value cannot be empty")
                .with_suggestion("Provide a non-empty value"));
        }
        Ok(())
    }

    /// Validate file extension
    pub fn validate_file_extension(
        path: &Path,
        expected_ext: &str,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext == expected_ext => Ok(()),
            Some(ext) => Err(ValidationError::new(
                field_name,
                format!("Expected .{} file, found .{}", expected_ext, ext),
            )
            .with_suggestion(format!("Use a .{} file", expected_ext))),
            None => Err(ValidationError::new(
                field_name,
                format!("File has no extension, expected .{}", expected_ext),
            )
            .with_suggestion(format!("Add .{} extension to the filename", expected_ext))),
        }
    }

    /// Validate that parent directory exists for a file path
    pub fn validate_parent_directory(path: &Path, field_name: &str) -> Result<(), ValidationError> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(ValidationError::new(
                    field_name,
                    format!("Parent directory does not exist: {}", parent.display()),
                )
                .with_suggestion(format!(
                    "Create parent directory: mkdir -p {}",
                    parent.display()
                )));
            }
        }
        Ok(())
    }
}

/// Configuration validation context
pub struct ValidationContext {
    pub base_path: PathBuf,
    pub strict_mode: bool,
}

impl ValidationContext {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            strict_mode: false,
        }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Resolve a path relative to the base path
    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_validate_url() {
        assert!(ValidationUtils::validate_url("http://localhost:5173", "base_url").is_ok());
        assert!(ValidationUtils::validate_url("https://example.com", "base_url").is_ok());
        assert!(ValidationUtils::validate_url("invalid-url", "base_url").is_err());
        assert!(ValidationUtils::validate_url("ftp://example.com", "base_url").is_err());
    }

    #[test]
    fn test_validate_url_error_messages() {
        let result = ValidationUtils::validate_url("invalid-url", "base_url");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field, "base_url");
        assert!(error.message.contains("Invalid URL format"));
        assert!(error
            .suggestion
            .as_ref()
            .unwrap()
            .contains("http://localhost:5173"));

        let result = ValidationUtils::validate_url("ftp://example.com", "base_url");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("must use http or https"));
    }

    #[test]
    fn test_validate_glob_pattern() {
        assert!(ValidationUtils::validate_glob_pattern("**/*.ts", "pattern").is_ok());
        assert!(ValidationUtils::validate_glob_pattern("app/**/*.cy.ts", "pattern").is_ok());
        assert!(ValidationUtils::validate_glob_pattern("[", "pattern").is_err());

        let result = ValidationUtils::validate_glob_pattern("[", "pattern");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field, "pattern");
        assert!(error.message.contains("Invalid glob pattern"));
        assert!(error
            .suggestion
            .as_ref()
            .unwrap()
            .contains("app/**/*.cy.ts"));
    }

    #[test]
    fn test_validate_numeric_range() {
        assert!(ValidationUtils::validate_numeric_range(5, 1, 10, "workers").is_ok());
        assert!(ValidationUtils::validate_numeric_range(0, 1, 10, "workers").is_err());
        assert!(ValidationUtils::validate_numeric_range(15, 1, 10, "workers").is_err());

        let result = ValidationUtils::validate_numeric_range(0, 1, 10, "workers");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field, "workers");
        assert!(error.message.contains("outside valid range [1, 10]"));
        assert!(error
            .suggestion
            .as_ref()
            .unwrap()
            .contains("between 1 and 10"));
    }

    #[test]
    fn test_validate_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");

        // Should fail if directory doesn't exist and creation is disabled
        assert!(ValidationUtils::validate_directory(&dir_path, "test_dir", false).is_err());

        // Should create directory if creation is enabled
        assert!(ValidationUtils::validate_directory(&dir_path, "test_dir", true).is_ok());
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());

        // Should succeed if directory already exists
        assert!(ValidationUtils::validate_directory(&dir_path, "test_dir", false).is_ok());
    }

    #[test]
    fn test_validate_directory_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a file at the path
        File::create(&file_path).unwrap();

        // Should fail if path exists but is not a directory
        let result = ValidationUtils::validate_directory(&file_path, "test_dir", true);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("not a directory"));
        assert!(error
            .suggestion
            .as_ref()
            .unwrap()
            .contains("Remove the file"));
    }

    #[test]
    fn test_validate_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Should fail if file doesn't exist
        let result = ValidationUtils::validate_file_exists(&file_path, "test_file");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field, "test_file");
        assert!(error.message.contains("does not exist"));
        assert!(error
            .suggestion
            .as_ref()
            .unwrap()
            .contains("Create the file"));

        // Create file and test again
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        assert!(ValidationUtils::validate_file_exists(&file_path, "test_file").is_ok());
    }

    #[test]
    fn test_validate_file_exists_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        std::fs::create_dir(&dir_path).unwrap();

        // Should fail if path exists but is not a file
        let result = ValidationUtils::validate_file_exists(&dir_path, "test_file");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("not a file"));
    }

    #[test]
    fn test_validate_file_extension() {
        let path = Path::new("config.json");
        assert!(ValidationUtils::validate_file_extension(path, "json", "config").is_ok());

        let result = ValidationUtils::validate_file_extension(path, "ts", "config");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Expected .ts file, found .json"));

        let path_no_ext = Path::new("config");
        let result = ValidationUtils::validate_file_extension(path_no_ext, "json", "config");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("no extension, expected .json"));
    }

    #[test]
    fn test_validate_non_empty_string() {
        assert!(ValidationUtils::validate_non_empty_string("valid", "field").is_ok());
        assert!(ValidationUtils::validate_non_empty_string("  valid  ", "field").is_ok());

        let result = ValidationUtils::validate_non_empty_string("", "field");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field, "field");
        assert!(error.message.contains("cannot be empty"));

        let result = ValidationUtils::validate_non_empty_string("   ", "field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let parent_dir = temp_dir.path().join("parent");
        let file_path = parent_dir.join("file.txt");

        // Should fail if parent doesn't exist
        let result = ValidationUtils::validate_parent_directory(&file_path, "file_path");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Parent directory does not exist"));
        assert!(error.suggestion.as_ref().unwrap().contains("mkdir -p"));

        // Create parent and test again
        std::fs::create_dir_all(&parent_dir).unwrap();
        assert!(ValidationUtils::validate_parent_directory(&file_path, "file_path").is_ok());
    }

    #[test]
    fn test_validation_context() {
        let temp_dir = TempDir::new().unwrap();
        let context = ValidationContext::new(temp_dir.path().to_path_buf()).with_strict_mode(true);

        assert_eq!(context.base_path, temp_dir.path());
        assert!(context.strict_mode);

        // Test path resolution
        let relative_path = Path::new("config/pulse.json");
        let resolved = context.resolve_path(relative_path);
        assert_eq!(resolved, temp_dir.path().join("config/pulse.json"));

        // Test absolute path (should remain unchanged)
        let absolute_path = Path::new("/absolute/path");
        let resolved = context.resolve_path(absolute_path);
        assert_eq!(resolved, absolute_path);
    }

    #[test]
    fn test_config_validator_trait() {
        struct TestConfig {
            url: String,
            timeout: u64,
        }

        impl ConfigValidator for TestConfig {
            fn validate(&self) -> Result<(), Vec<ValidationError>> {
                let mut errors = Vec::new();

                if let Err(e) = ValidationUtils::validate_url(&self.url, "url") {
                    errors.push(e);
                }

                if let Err(e) =
                    ValidationUtils::validate_numeric_range(self.timeout, 1000, 60000, "timeout")
                {
                    errors.push(e);
                }

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }

        let valid_config = TestConfig {
            url: "http://localhost:5173".to_string(),
            timeout: 5000,
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = TestConfig {
            url: "invalid-url".to_string(),
            timeout: 500,
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.field == "url"));
        assert!(errors.iter().any(|e| e.field == "timeout"));
    }
}
