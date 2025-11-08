use crate::utils::FileSystemUtils;
use crate::ApicentricResult;
use log::debug;
use std::path::PathBuf;

/// Scans directories for files matching patterns.
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
    pub fn scan_for_files(&self, pattern: &str) -> ApicentricResult<Vec<PathBuf>> {
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
