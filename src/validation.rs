//! Provides validation utilities for configuration objects and common data types.
//!
//! This module includes a `ConfigValidator` trait for validating configuration
//! objects, and a `ValidationUtils` struct with a collection of static methods
//! for common validation tasks, such as checking if a path exists, validating
//! a URL, or ensuring a string is not empty.

use crate::errors::ValidationError;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// A trait for validating configuration objects.
///
/// This trait provides a common interface for validating configuration objects.
/// It is implemented by configuration structs to provide custom validation logic.
pub trait ConfigValidator {
    /// Validates the configuration object.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful. If the
    /// validation fails, it returns a `Vec` of `ValidationError`s.
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
}

/// A collection of utility functions for common validation tasks.
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validates that a path exists and is accessible.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that a directory exists. Optionally creates it if it is missing.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory.
    /// * `field_name` - The name of the field being validated.
    /// * `create_if_missing` - Whether to create the directory if it is missing.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that a file exists and is readable.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that a string is a valid URL.
    ///
    /// # Arguments
    ///
    /// * `url_str` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that a string is a valid glob pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_glob_pattern(pattern: &str, field_name: &str) -> Result<(), ValidationError> {
        match glob::Pattern::new(pattern) {
            Ok(_) => Ok(()),
            Err(e) => Err(
                ValidationError::new(field_name, format!("Invalid glob pattern: {}", e))
                    .with_suggestion("Check glob pattern syntax, e.g., 'app/**/*.cy.ts'"),
            ),
        }
    }

    /// Validates that a numeric value is within a given range.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate.
    /// * `min` - The minimum allowed value.
    /// * `max` - The maximum allowed value.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that a string is not empty.
    ///
    /// # Arguments
    ///
    /// * `value` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::new(field_name, "Value cannot be empty")
                .with_suggestion("Provide a non-empty value"));
        }
        Ok(())
    }

    /// Validates that a file has a specific extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `expected_ext` - The expected file extension.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that the parent directory of a file path exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
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

    /// Validates that a port number is in the valid range.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_port(port: u16, field_name: &str) -> Result<(), ValidationError> {
        if port < 1024 {
            return Err(ValidationError::new(
                field_name,
                format!("Port {} is below 1024 (system port range)", port),
            )
            .with_suggestion("Use ports 1024 or higher to avoid conflicts with system services"));
        }
        Ok(())
    }

    /// Validates that a port range is valid.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the port range.
    /// * `end` - The end of the port range.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_port_range(
        start: u16,
        end: u16,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        if start >= end {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid port range: start ({}) must be less than end ({})", start, end),
            )
            .with_suggestion("Ensure start port is less than end port, e.g., 8000-8999"));
        }

        if start < 1024 {
            return Err(ValidationError::new(
                field_name,
                format!("Port range starts at {} (system port range)", start),
            )
            .with_suggestion("Use ports 1024 or higher to avoid conflicts with system services"));
        }

        let range_size = end - start;
        if range_size < 10 {
            return Err(ValidationError::new(
                field_name,
                format!("Port range is too small ({} ports)", range_size),
            )
            .with_suggestion("Provide at least 10 ports in the range for flexibility"));
        }

        Ok(())
    }

    /// Validates that a string is a valid HTTP method.
    ///
    /// # Arguments
    ///
    /// * `method` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_http_method(method: &str, field_name: &str) -> Result<(), ValidationError> {
        let valid_methods = [
            "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CONNECT", "TRACE",
        ];

        if !valid_methods.contains(&method.to_uppercase().as_str()) {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid HTTP method: {}", method),
            )
            .with_suggestion(format!(
                "Use one of: {}",
                valid_methods.join(", ")
            )));
        }
        Ok(())
    }

    /// Validates that a number is a valid HTTP status code.
    ///
    /// # Arguments
    ///
    /// * `status` - The number to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_status_code(status: u16, field_name: &str) -> Result<(), ValidationError> {
        if !(100..=599).contains(&status) {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid HTTP status code: {}", status),
            )
            .with_suggestion("Use a valid HTTP status code between 100 and 599"));
        }
        Ok(())
    }

    /// Validates that a string is a valid content type.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_content_type(
        content_type: &str,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        // Basic validation - check for slash
        if !content_type.contains('/') {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid content type format: {}", content_type),
            )
            .with_suggestion("Use format: type/subtype, e.g., 'application/json' or 'text/html'"));
        }

        let parts: Vec<&str> = content_type.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid content type format: {}", content_type),
            )
            .with_suggestion("Use format: type/subtype, e.g., 'application/json' or 'text/html'"));
        }

        Ok(())
    }

    /// Validates that a string is a valid JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_json_string(json: &str, field_name: &str) -> Result<(), ValidationError> {
        if let Err(e) = serde_json::from_str::<serde_json::Value>(json) {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid JSON: {}", e),
            )
            .with_suggestion("Ensure the JSON is properly formatted with valid syntax"));
        }
        Ok(())
    }

    /// Validates that a string is a valid YAML string.
    ///
    /// # Arguments
    ///
    /// * `yaml` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_yaml_string(yaml: &str, field_name: &str) -> Result<(), ValidationError> {
        if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(yaml) {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid YAML: {}", e),
            )
            .with_suggestion("Ensure the YAML is properly formatted with correct indentation"));
        }
        Ok(())
    }

    /// Validates that a string is a valid service name.
    ///
    /// # Arguments
    ///
    /// * `name` - The string to validate.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_service_name(name: &str, field_name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(field_name, "Service name cannot be empty")
                .with_suggestion("Provide a descriptive service name"));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ValidationError::new(
                field_name,
                format!("Invalid service name: {}", name),
            )
            .with_suggestion(
                "Use only alphanumeric characters, hyphens, and underscores (e.g., 'user-api', 'auth_service')",
            ));
        }

        if name.starts_with('-') || name.starts_with('_') {
            return Err(ValidationError::new(
                field_name,
                format!("Service name cannot start with hyphen or underscore: {}", name),
            )
            .with_suggestion("Start the service name with an alphanumeric character"));
        }

        Ok(())
    }

    /// Validates that a timeout value is within a reasonable range.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - The timeout value in milliseconds.
    /// * `field_name` - The name of the field being validated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the validation was successful.
    pub fn validate_timeout(timeout_ms: u64, field_name: &str) -> Result<(), ValidationError> {
        if timeout_ms < 100 {
            return Err(ValidationError::new(
                field_name,
                format!("Timeout {} ms is too short", timeout_ms),
            )
            .with_suggestion("Use a timeout of at least 100 ms"));
        }

        if timeout_ms > 300_000 {
            return Err(ValidationError::new(
                field_name,
                format!("Timeout {} ms is too long", timeout_ms),
            )
            .with_suggestion("Use a timeout of 300000 ms (5 minutes) or less"));
        }

        Ok(())
    }
}

/// The context for a validation operation.
pub struct ValidationContext {
    /// The base path for resolving relative paths.
    pub base_path: PathBuf,
    /// Whether to use strict mode for validation.
    pub strict_mode: bool,
}

impl ValidationContext {
    /// Creates a new validation context.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path for resolving relative paths.
    ///
    /// # Returns
    ///
    /// A new `ValidationContext` instance.
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            strict_mode: false,
        }
    }

    /// Sets the strict mode for the validation context.
    ///
    /// # Arguments
    ///
    /// * `strict` - Whether to use strict mode.
    ///
    /// # Returns
    ///
    /// The `ValidationContext` instance with the strict mode set.
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Resolves a path relative to the base path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to resolve.
    ///
    /// # Returns
    ///
    /// The resolved path.
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
        let relative_path = Path::new("config/apicentric.json");
        let resolved = context.resolve_path(relative_path);
        assert_eq!(resolved, temp_dir.path().join("config/apicentric.json"));

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

    #[test]
    fn test_validate_port() {
        assert!(ValidationUtils::validate_port(8080, "port").is_ok());
        assert!(ValidationUtils::validate_port(1024, "port").is_ok());
        
        let result = ValidationUtils::validate_port(80, "port");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("below 1024"));
        assert!(error.suggestion.as_ref().unwrap().contains("1024 or higher"));
    }

    #[test]
    fn test_validate_port_range() {
        assert!(ValidationUtils::validate_port_range(8000, 8999, "port_range").is_ok());
        
        // Invalid: start >= end
        let result = ValidationUtils::validate_port_range(9000, 8000, "port_range");
        assert!(result.is_err());
        
        // Invalid: too small range
        let result = ValidationUtils::validate_port_range(8000, 8005, "port_range");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("too small"));
        
        // Invalid: system ports
        let result = ValidationUtils::validate_port_range(80, 100, "port_range");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_http_method() {
        assert!(ValidationUtils::validate_http_method("GET", "method").is_ok());
        assert!(ValidationUtils::validate_http_method("post", "method").is_ok());
        assert!(ValidationUtils::validate_http_method("DELETE", "method").is_ok());
        assert!(ValidationUtils::validate_http_method("CONNECT", "method").is_ok());
        assert!(ValidationUtils::validate_http_method("trace", "method").is_ok());

        let result = ValidationUtils::validate_http_method("INVALID", "method");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid HTTP method"));
        assert!(error.suggestion.as_ref().unwrap().contains("GET"));
    }

    #[test]
    fn test_validate_status_code() {
        assert!(ValidationUtils::validate_status_code(200, "status").is_ok());
        assert!(ValidationUtils::validate_status_code(404, "status").is_ok());
        assert!(ValidationUtils::validate_status_code(500, "status").is_ok());
        
        let result = ValidationUtils::validate_status_code(99, "status");
        assert!(result.is_err());
        
        let result = ValidationUtils::validate_status_code(600, "status");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_content_type() {
        assert!(ValidationUtils::validate_content_type("application/json", "content_type").is_ok());
        assert!(ValidationUtils::validate_content_type("text/html", "content_type").is_ok());
        
        let result = ValidationUtils::validate_content_type("invalid", "content_type");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid content type"));
        
        let result = ValidationUtils::validate_content_type("application/", "content_type");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_string() {
        assert!(ValidationUtils::validate_json_string(r#"{"key": "value"}"#, "json").is_ok());
        assert!(ValidationUtils::validate_json_string("[]", "json").is_ok());
        assert!(ValidationUtils::validate_json_string("null", "json").is_ok());
        
        let result = ValidationUtils::validate_json_string("{invalid json}", "json");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid JSON"));
    }

    #[test]
    fn test_validate_yaml_string() {
        assert!(ValidationUtils::validate_yaml_string("key: value", "yaml").is_ok());
        assert!(ValidationUtils::validate_yaml_string("- item1\n- item2", "yaml").is_ok());
        
        let result = ValidationUtils::validate_yaml_string("invalid: yaml: :", "yaml");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid YAML"));
    }

    #[test]
    fn test_validate_service_name() {
        assert!(ValidationUtils::validate_service_name("user-api", "name").is_ok());
        assert!(ValidationUtils::validate_service_name("auth_service", "name").is_ok());
        assert!(ValidationUtils::validate_service_name("api123", "name").is_ok());
        
        let result = ValidationUtils::validate_service_name("", "name");
        assert!(result.is_err());
        
        let result = ValidationUtils::validate_service_name("invalid name", "name");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid service name"));
        
        let result = ValidationUtils::validate_service_name("-invalid", "name");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("cannot start with"));
    }

    #[test]
    fn test_validate_timeout() {
        assert!(ValidationUtils::validate_timeout(1000, "timeout").is_ok());
        assert!(ValidationUtils::validate_timeout(30000, "timeout").is_ok());
        
        let result = ValidationUtils::validate_timeout(50, "timeout");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("too short"));
        
        let result = ValidationUtils::validate_timeout(400_000, "timeout");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("too long"));
    }
}
