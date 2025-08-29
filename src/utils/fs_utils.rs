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

/// Directory scanner with enhanced error handling
pub struct DirectoryScanner {
    base_path: PathBuf,
    max_depth: Option<usize>,
    follow_symlinks: bool,
}

impl DirectoryScanner {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            max_depth: None,
            follow_symlinks: false,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Scan directory recursively with error resilience
    pub fn scan_for_files(&self, pattern: &str) -> PulseResult<Vec<PathBuf>> {
        FileSystemUtils::validate_directory_access(&self.base_path)?;

        let full_pattern = self.base_path.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        debug!(
            "Scanning directory {} with pattern: {}",
            self.base_path.display(),
            pattern
        );

        FileSystemUtils::resolve_glob_pattern(&pattern_str, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create test directories
        fs::create_dir_all(base.join("app/routes/login/test")).unwrap();
        fs::create_dir_all(base.join("app/routes/dashboard/test")).unwrap();
        fs::create_dir_all(base.join("cypress/screenshots")).unwrap();
        fs::create_dir_all(base.join("cypress/videos")).unwrap();

        // Create test files
        File::create(base.join("app/routes/login/test/login.cy.ts")).unwrap();
        File::create(base.join("app/routes/dashboard/test/dashboard.cy.ts")).unwrap();
        File::create(base.join("cypress/screenshots/test.png")).unwrap();
        File::create(base.join("cypress/videos/test.mp4")).unwrap();
        File::create(base.join("app/routes/login/route.tsx")).unwrap();

        temp_dir
    }

    #[test]
    fn test_resolve_glob_pattern_success() {
        let temp_dir = create_test_structure();
        let pattern = format!("{}/**/test/*.cy.ts", temp_dir.path().display());

        let result = FileSystemUtils::resolve_glob_pattern(&pattern, None);
        assert!(result.is_ok());

        let paths = result.unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("login.cy.ts")));
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("dashboard.cy.ts")));
    }

    #[test]
    fn test_resolve_glob_pattern_invalid_syntax() {
        let result = FileSystemUtils::resolve_glob_pattern("[invalid", None);
        assert!(result.is_err());

        if let Err(PulseError::FileSystem {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("Invalid glob pattern syntax"));
            assert!(suggestion.is_some());
        } else {
            panic!("Expected FileSystem error");
        }
    }

    #[test]
    fn test_resolve_glob_pattern_no_matches() {
        let temp_dir = create_test_structure();
        let pattern = format!("{}/**/nonexistent/*.cy.ts", temp_dir.path().display());

        let result = FileSystemUtils::resolve_glob_pattern(&pattern, None);
        assert!(result.is_err());

        if let Err(PulseError::FileSystem {
            message,
            suggestion,
        }) = result
        {
            assert!(message.contains("No valid files found"));
            assert!(suggestion.is_some());
        } else {
            panic!("Expected FileSystem error");
        }
    }

    #[test]
    fn test_validate_test_files() {
        let temp_dir = create_test_structure();
        let paths = vec![
            temp_dir.path().join("app/routes/login/test/login.cy.ts"),
            temp_dir.path().join("cypress/screenshots/test.png"),
            temp_dir.path().join("app/routes/login/route.tsx"),
        ];

        let (valid_files, issues) = FileSystemUtils::validate_test_files(&paths);

        assert_eq!(valid_files.len(), 1);
        assert_eq!(issues.len(), 2);
        assert!(valid_files[0].to_string_lossy().contains("login.cy.ts"));
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_single_test_file_posix() {
        let temp_dir = create_test_structure();

        // Valid test file
        let valid_file = temp_dir.path().join("app/routes/login/test/login.cy.ts");
        assert!(FileSystemUtils::validate_single_test_file(&valid_file).is_ok());

        // Invalid extension
        let invalid_file = temp_dir.path().join("app/routes/login/route.tsx");
        assert!(FileSystemUtils::validate_single_test_file(&invalid_file).is_err());

        // Excluded directory
        let excluded_file = temp_dir.path().join("cypress/screenshots/test.png");
        assert!(FileSystemUtils::validate_single_test_file(&excluded_file).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn test_validate_single_test_file_windows() {
        let temp_dir = create_test_structure();

        // Valid test file
        let valid_file = temp_dir
            .path()
            .join("app\\routes\\login\\test\\login.cy.ts");
        assert!(FileSystemUtils::validate_single_test_file(&valid_file).is_ok());

        // Invalid extension
        let invalid_file = temp_dir.path().join("app\\routes\\login\\route.tsx");
        assert!(FileSystemUtils::validate_single_test_file(&invalid_file).is_err());

        // Excluded directory
        let excluded_file = temp_dir.path().join("cypress\\screenshots\\test.png");
        assert!(FileSystemUtils::validate_single_test_file(&excluded_file).is_err());
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new/nested/directory");

        assert!(!new_dir.exists());

        let result = FileSystemUtils::ensure_directory_exists(&new_dir);
        assert!(result.is_ok());
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_safe_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        let result = FileSystemUtils::safe_read_file(&file_path);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test content"));
    }

    #[test]
    fn test_safe_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested/test.txt");

        let result = FileSystemUtils::safe_write_file(&file_path, "test content");
        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_validate_directory_access() {
        let temp_dir = create_test_structure();

        // Valid directory
        let valid_dir = temp_dir.path().join("app");
        assert!(FileSystemUtils::validate_directory_access(&valid_dir).is_ok());

        // Non-existent directory
        let invalid_dir = temp_dir.path().join("nonexistent");
        assert!(FileSystemUtils::validate_directory_access(&invalid_dir).is_err());

        // File instead of directory
        let file_path = temp_dir.path().join("app/routes/login/route.tsx");
        assert!(FileSystemUtils::validate_directory_access(&file_path).is_err());
    }

    #[test]
    fn test_directory_scanner() {
        let temp_dir = create_test_structure();
        let scanner = DirectoryScanner::new(temp_dir.path().to_path_buf());

        let result = scanner.scan_for_files("**/test/*.cy.ts");
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_count_files_in_directory() {
        let temp_dir = create_test_structure();
        let test_dir = temp_dir.path().join("app/routes/login/test");

        let result = FileSystemUtils::count_files_in_directory(&test_dir, Some(".cy.ts"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
