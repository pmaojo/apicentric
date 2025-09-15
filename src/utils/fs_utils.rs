use crate::{PulseError, PulseResult};
use log::{debug, error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

/// File system utilities with enhanced error handling and validation
pub struct FileSystemUtils;

impl FileSystemUtils {
    /// Safely resolve and validate a glob pattern with comprehensive error handling
    pub fn resolve_glob_pattern(
        pattern: &str,
        base_path: Option<&Path>,
    ) -> PulseResult<Vec<PathBuf>> {
        debug!("Resolving glob pattern: {}", pattern);

        // Validate the glob pattern syntax first
        if let Err(e) = glob::Pattern::new(pattern) {
            return Err(PulseError::fs_error(
                format!("Invalid glob pattern syntax: {}", e),
                Some("Check your glob pattern syntax. Examples: '**/*.cy.ts', 'app/routes/**/test/*.cy.ts'")
            ));
        }

        // Resolve pattern relative to base path if provided
        let resolved_pattern = if let Some(base) = base_path {
            if Path::new(pattern).is_absolute() {
                pattern.to_string()
            } else {
                base.join(pattern).to_string_lossy().into_owned()
            }
        } else {
            pattern.to_string()
        };

        debug!("Using resolved pattern: {}", resolved_pattern);

        // Execute glob with error handling
        let glob_entries = match glob::glob(&resolved_pattern) {
            Ok(entries) => entries,
            Err(e) => {
                error!(
                    "Glob execution failed for pattern '{}': {}",
                    resolved_pattern, e
                );
                return Err(PulseError::fs_error(
                    format!("Failed to execute glob pattern '{}': {}", pattern, e),
                    Some("Verify the pattern syntax and ensure the base directory exists"),
                ));
            }
        };

        let mut valid_paths = Vec::new();
        let mut errors = Vec::new();

        // Process glob results with individual error handling
        for entry_result in glob_entries {
            match entry_result {
                Ok(path) => {
                    // Validate each path
                    match Self::validate_path_safety(&path) {
                        Ok(()) => {
                            debug!("Valid path found: {}", path.display());
                            valid_paths.push(path);
                        }
                        Err(e) => {
                            warn!("Skipping unsafe path {}: {}", path.display(), e);
                            errors.push(format!("Skipped {}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    warn!("Glob entry error: {}", e);
                    errors.push(format!("Glob entry error: {}", e));
                }
            }
        }

        // Log summary of errors if any
        if !errors.is_empty() {
            info!(
                "Encountered {} path issues during glob resolution:",
                errors.len()
            );
            for error in &errors {
                info!("  - {}", error);
            }
        }

        // Provide helpful feedback if no valid paths found
        if valid_paths.is_empty() {
            let suggestion = Self::generate_no_files_suggestion(pattern, base_path);
            return Err(PulseError::fs_error(
                format!("No valid files found matching pattern: {}", pattern),
                Some(suggestion),
            ));
        }

        info!(
            "Found {} valid files matching pattern '{}'",
            valid_paths.len(),
            pattern
        );
        Ok(valid_paths)
    }

    /// Validate that a path is safe to access (no symlinks, within bounds, etc.)
    pub fn validate_path_safety(path: &Path) -> PulseResult<()> {
        // Check if path exists
        if !path.exists() {
            return Err(PulseError::fs_error(
                format!("Path does not exist: {}", path.display()),
                None::<String>,
            ));
        }

        // Skip symbolic links for security
        if path.is_symlink() {
            return Err(PulseError::fs_error(
                format!("Skipping symbolic link for security: {}", path.display()),
                Some("Symbolic links are not followed for security reasons"),
            ));
        }

        // Check for suspicious paths
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(PulseError::fs_error(
                format!(
                    "Path contains suspicious '..' components: {}",
                    path.display()
                ),
                Some("Use absolute paths or ensure paths don't traverse parent directories"),
            ));
        }

        // Check file permissions
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    debug!("Path is read-only: {}", path.display());
                }
                Ok(())
            }
            Err(e) => Err(PulseError::fs_error(
                format!("Cannot access path metadata {}: {}", path.display(), e),
                Some("Check file permissions and ensure the path is accessible"),
            )),
        }
    }

    /// Validate and filter test files with comprehensive checks
    pub fn validate_test_files(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>) {
        let mut valid_files = Vec::new();
        let mut issues = Vec::new();

        for path in paths {
            match Self::validate_single_test_file(path) {
                Ok(()) => {
                    valid_files.push(path.clone());
                }
                Err(issue) => {
                    issues.push(format!("{}: {}", path.display(), issue));
                }
            }
        }

        (valid_files, issues)
    }

    /// Validate a single test file
    pub fn validate_single_test_file(path: &Path) -> Result<(), String> {
        // Check if it's actually a file
        if !path.is_file() {
            return Err("Not a regular file".to_string());
        }

        // Check file extension
        let valid_extensions = [".cy.ts", ".cy.js"];
        let extension = path.extension().and_then(|ext| ext.to_str());
        let stem = path.file_stem().and_then(|s| s.to_str());

        let valid = matches!(extension, Some("ts" | "js"))
            && stem.map(|s| s.ends_with(".cy")).unwrap_or(false);

        if !valid {
            return Err(format!(
                "Invalid test file extension. Expected one of: {}",
                valid_extensions.join(", ")
            ));
        }

        // Skip non-test directories
        let excluded_dirs = ["screenshots", "videos", "node_modules", "dist", "build"];
        if excluded_dirs.iter().any(|dir| {
            let dir_os = std::ffi::OsStr::new(dir);
            path.starts_with(dir) || path.components().any(|c| c.as_os_str() == dir_os)
        }) {
            return Err("File is in excluded directory".to_string());
        }

        // Check file size (avoid extremely large files)
        match fs::metadata(path) {
            Ok(metadata) => {
                const MAX_TEST_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
                if metadata.len() > MAX_TEST_FILE_SIZE {
                    return Err(format!(
                        "File too large ({} bytes). Max size: {} bytes",
                        metadata.len(),
                        MAX_TEST_FILE_SIZE
                    ));
                }
            }
            Err(e) => {
                return Err(format!("Cannot read file metadata: {}", e));
            }
        }

        // Test file readability
        match fs::File::open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cannot read file: {}", e)),
        }
    }

    /// Generate helpful suggestions when no files are found
    pub fn generate_no_files_suggestion(pattern: &str, base_path: Option<&Path>) -> String {
        let mut suggestions = Vec::new();

        // Check if base path exists
        if let Some(base) = base_path {
            if !base.exists() {
                suggestions.push(format!("Base directory does not exist: {}", base.display()));
                suggestions.push(format!("Create it with: mkdir -p {}", base.display()));
            }
        }

        // Provide pattern-specific suggestions
        if pattern.contains("**") {
            suggestions.push(
                "The pattern uses recursive matching (**). Ensure subdirectories exist."
                    .to_string(),
            );
        }

        if pattern.contains(".cy.ts") || pattern.contains(".cy.js") {
            suggestions.push(
                "Looking for Cypress test files. Ensure they have .cy.ts or .cy.js extensions."
                    .to_string(),
            );
        }

        if pattern.contains("/test/") || pattern.contains("/__tests__/") {
            suggestions
                .push("Pattern expects files in 'test' or '__tests__' directories.".to_string());
        }

        // General suggestions
        suggestions.push("Check that:".to_string());
        suggestions.push("  - Test files exist in the expected locations".to_string());
        suggestions.push("  - File permissions allow reading".to_string());
        suggestions.push("  - The glob pattern matches your directory structure".to_string());

        suggestions.join("\n")
    }

    /// Safely create directory with proper error handling
    pub fn ensure_directory_exists(path: &Path) -> PulseResult<()> {
        if path.exists() {
            if !path.is_dir() {
                return Err(PulseError::fs_error(
                    format!("Path exists but is not a directory: {}", path.display()),
                    Some("Remove the file or choose a different path"),
                ));
            }
            debug!("Directory already exists: {}", path.display());
            return Ok(());
        }

        debug!("Creating directory: {}", path.display());
        fs::create_dir_all(path).map_err(|e| {
            PulseError::fs_error(
                format!("Failed to create directory {}: {}", path.display(), e),
                Some("Check parent directory permissions and available disk space"),
            )
        })?;

        info!("Created directory: {}", path.display());
        Ok(())
    }

    /// Safely read file with size and permission checks
    pub fn safe_read_file(path: &Path) -> PulseResult<String> {
        // Validate path safety first
        Self::validate_path_safety(path)?;

        // Check if it's a file
        if !path.is_file() {
            return Err(PulseError::fs_error(
                format!("Path is not a file: {}", path.display()),
                None::<String>,
            ));
        }

        // Check file size
        let metadata = fs::metadata(path).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read file metadata {}: {}", path.display(), e),
                Some("Check file permissions"),
            )
        })?;

        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
        if metadata.len() > MAX_FILE_SIZE {
            return Err(PulseError::fs_error(
                format!(
                    "File too large to read: {} bytes (max: {} bytes)",
                    metadata.len(),
                    MAX_FILE_SIZE
                ),
                Some("Consider processing the file in chunks or increasing the size limit"),
            ));
        }

        // Read file content
        fs::read_to_string(path).map_err(|e| {
            PulseError::fs_error(
                format!("Failed to read file {}: {}", path.display(), e),
                Some("Check file permissions and encoding (must be UTF-8)"),
            )
        })
    }

    /// Safely write file with backup and atomic operations
    pub fn safe_write_file(path: &Path, content: &str) -> PulseResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            Self::ensure_directory_exists(parent)?;
        }

        // Create temporary file for atomic write
        let temp_path = path.with_extension("tmp");

        // Write to temporary file first
        fs::write(&temp_path, content).map_err(|e| {
            PulseError::fs_error(
                format!(
                    "Failed to write temporary file {}: {}",
                    temp_path.display(),
                    e
                ),
                Some("Check directory permissions and available disk space"),
            )
        })?;

        // Atomically move temporary file to final location
        fs::rename(&temp_path, path).map_err(|e| {
            // Clean up temporary file on failure
            let _ = fs::remove_file(&temp_path);
            PulseError::fs_error(
                format!(
                    "Failed to move temporary file to final location {}: {}",
                    path.display(),
                    e
                ),
                Some("Check directory permissions"),
            )
        })?;

        debug!("Successfully wrote file: {}", path.display());
        Ok(())
    }

    /// Check if directory is accessible and readable
    pub fn validate_directory_access(path: &Path) -> PulseResult<()> {
        if !path.exists() {
            return Err(PulseError::fs_error(
                format!("Directory does not exist: {}", path.display()),
                Some(format!(
                    "Create the directory with: mkdir -p {}",
                    path.display()
                )),
            ));
        }

        if !path.is_dir() {
            return Err(PulseError::fs_error(
                format!("Path is not a directory: {}", path.display()),
                Some("Ensure the path points to a directory, not a file"),
            ));
        }

        // Test directory readability
        match fs::read_dir(path) {
            Ok(_) => {
                debug!("Directory is accessible: {}", path.display());
                Ok(())
            }
            Err(e) => Err(PulseError::fs_error(
                format!("Cannot read directory {}: {}", path.display(), e),
                Some("Check directory permissions"),
            )),
        }
    }

    /// Get file count in directory with pattern matching
    pub fn count_files_in_directory(dir: &Path, pattern: Option<&str>) -> PulseResult<usize> {
        Self::validate_directory_access(dir)?;

        let entries = fs::read_dir(dir).map_err(|e| {
            PulseError::fs_error(
                format!("Failed to read directory {}: {}", dir.display(), e),
                None::<String>,
            )
        })?;

        let mut count = 0;
        for entry_result in entries {
            match entry_result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(pat) = pattern {
                            if path.to_string_lossy().contains(pat) {
                                count += 1;
                            }
                        } else {
                            count += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading directory entry: {}", e);
                }
            }
        }

        Ok(count)
    }
}
