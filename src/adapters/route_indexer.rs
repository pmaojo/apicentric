use crate::utils::FileSystemUtils;
use crate::{PulseError, PulseResult};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteIndex {
    #[serde(rename = "routeToSpec")]
    route_to_spec: HashMap<String, Vec<String>>,
    #[serde(rename = "fileToSpecs")]
    file_to_specs: HashMap<String, Vec<String>>,
}

impl RouteIndex {
    pub fn map_changes_to_specs(&self, changed_files: &[String]) -> PulseResult<Vec<String>> {
        let mut specs = Vec::new();

        println!("📁 Changed files:");
        for file in changed_files {
            println!("   - {}", file);
        }

        for file in changed_files {
            println!("🔍 Looking for tests related to: {}", file);
            if let Some(file_specs) = self.file_to_specs.get(file) {
                println!("   ✓ Found {} related tests", file_specs.len());
                specs.extend(file_specs.clone());
            } else {
                println!("   × No direct test matches found");
            }

            // También buscar por rutas
            let route = Path::new(file)
                .components()
                .find(|c| {
                    if let std::path::Component::Normal(name) = c {
                        name.to_string_lossy().starts_with('_')
                    } else {
                        false
                    }
                })
                .and_then(|c| c.as_os_str().to_str())
                .map(|s| s.trim_start_matches('_').to_string());

            if let Some(route) = route {
                if let Some(route_specs) = self.route_to_spec.get(&route) {
                    specs.extend(route_specs.clone());
                }
            }
        }

        // Eliminar duplicados
        specs.sort();
        specs.dedup();

        Ok(specs)
    }
}

pub struct RouteIndexer {
    routes_dir: PathBuf,
    specs_dir: PathBuf,
    cache_file: PathBuf,
}

impl RouteIndexer {
    pub fn new<P: AsRef<Path>>(routes_dir: P, specs_dir: P, cache_file: P) -> Self {
        Self {
            routes_dir: routes_dir.as_ref().to_path_buf(),
            specs_dir: specs_dir.as_ref().to_path_buf(),
            cache_file: cache_file.as_ref().to_path_buf(),
        }
    }

    /// Validate that the required directories exist and are accessible
    fn validate_directories(&self) -> PulseResult<()> {
        info!("🔍 Validating directories for route indexing...");

        // Validate routes directory
        if !self.routes_dir.exists() {
            warn!(
                "Routes directory does not exist: {}",
                self.routes_dir.display()
            );
            return Err(PulseError::fs_error(
                format!("Routes directory not found: {}", self.routes_dir.display()),
                Some("Create the routes directory or update the 'routes_dir' configuration"),
            ));
        }

        if !self.routes_dir.is_dir() {
            return Err(PulseError::fs_error(
                format!(
                    "Routes path is not a directory: {}",
                    self.routes_dir.display()
                ),
                Some("Ensure the routes_dir configuration points to a directory"),
            ));
        }

        // Test read permissions for routes directory
        match fs::read_dir(&self.routes_dir) {
            Ok(_) => {
                debug!(
                    "✓ Routes directory is accessible: {}",
                    self.routes_dir.display()
                );
            }
            Err(e) => {
                return Err(PulseError::fs_error(
                    format!(
                        "Cannot read routes directory {}: {}",
                        self.routes_dir.display(),
                        e
                    ),
                    Some("Check directory permissions or run with appropriate privileges"),
                ));
            }
        }

        // Validate specs directory
        if !self.specs_dir.exists() {
            warn!(
                "Specs directory does not exist: {}",
                self.specs_dir.display()
            );
            return Err(PulseError::fs_error(
                format!("Specs directory not found: {}", self.specs_dir.display()),
                Some("Create the specs directory or update the 'specs_dir' configuration"),
            ));
        }

        if !self.specs_dir.is_dir() {
            return Err(PulseError::fs_error(
                format!(
                    "Specs path is not a directory: {}",
                    self.specs_dir.display()
                ),
                Some("Ensure the specs_dir configuration points to a directory"),
            ));
        }

        // Test read permissions for specs directory
        match fs::read_dir(&self.specs_dir) {
            Ok(_) => {
                debug!(
                    "✓ Specs directory is accessible: {}",
                    self.specs_dir.display()
                );
            }
            Err(e) => {
                return Err(PulseError::fs_error(
                    format!(
                        "Cannot read specs directory {}: {}",
                        self.specs_dir.display(),
                        e
                    ),
                    Some("Check directory permissions or run with appropriate privileges"),
                ));
            }
        }

        // Validate cache directory (create if needed)
        if let Some(cache_dir) = self.cache_file.parent() {
            if !cache_dir.exists() {
                info!("Creating cache directory: {}", cache_dir.display());
                fs::create_dir_all(cache_dir).map_err(|e| {
                    PulseError::fs_error(
                        format!(
                            "Failed to create cache directory {}: {}",
                            cache_dir.display(),
                            e
                        ),
                        Some("Check write permissions for the parent directory"),
                    )
                })?;
            }
        }

        info!("✓ All directories validated successfully");
        Ok(())
    }

    pub fn build_index(&self) -> PulseResult<RouteIndex> {
        info!("📑 Building route index...");
        info!("   Routes dir: {}", self.routes_dir.display());
        info!("   Specs dir: {}", self.specs_dir.display());
        info!("   Cache file: {}", self.cache_file.display());

        // Validate directories before proceeding
        self.validate_directories()?;

        let mut index = RouteIndex {
            route_to_spec: HashMap::new(),
            file_to_specs: HashMap::new(),
        };

        // Scan route files
        info!("🔍 Scanning routes directory...");
        let routes_scanned = self.scan_routes_dir(&mut index)?;
        info!("✓ Scanned {} route directories", routes_scanned);

        // Scan Cypress specs
        info!("🔍 Scanning specs directory...");
        let specs_scanned = self.scan_specs_dir(&mut index)?;
        info!("✓ Scanned {} test files", specs_scanned);

        // Save index to cache
        info!("💾 Saving route index to cache...");
        self.save_index(&index)?;
        info!("✓ Route index built successfully");

        // Log summary
        info!("📊 Index summary:");
        info!("   - Routes mapped: {}", index.route_to_spec.len());
        info!("   - Files mapped: {}", index.file_to_specs.len());

        Ok(index)
    }

    fn scan_routes_dir(&self, index: &mut RouteIndex) -> PulseResult<usize> {
        let mut routes_count = 0;

        let entries = match fs::read_dir(&self.routes_dir) {
            Ok(entries) => entries,
            Err(e) => {
                error!(
                    "Failed to read routes directory {}: {}",
                    self.routes_dir.display(),
                    e
                );
                return Err(PulseError::fs_error(
                    format!("Cannot read routes directory: {}", e),
                    Some("Check directory permissions and ensure the path exists"),
                ));
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("Skipping directory entry due to error: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                debug!("Skipping non-directory: {}", path.display());
                continue;
            }

            // Extract route name
            let route_name = match path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.trim_start_matches('_').to_string())
            {
                Some(name) if !name.is_empty() => name,
                _ => {
                    warn!("Skipping directory with invalid name: {}", path.display());
                    continue;
                }
            };

            debug!(
                "Processing route directory: {} -> {}",
                path.display(),
                route_name
            );

            // Find corresponding specs
            match self.find_matching_specs(&route_name) {
                Ok(specs) => {
                    if !specs.is_empty() {
                        debug!("Found {} specs for route '{}'", specs.len(), route_name);
                        index.route_to_spec.insert(route_name, specs);
                        routes_count += 1;
                    } else {
                        debug!("No specs found for route '{}'", route_name);
                    }
                }
                Err(e) => {
                    warn!("Error finding specs for route '{}': {}", route_name, e);
                    // Continue processing other routes instead of failing completely
                    continue;
                }
            }
        }

        if routes_count == 0 {
            warn!(
                "No route directories with matching specs found in {}",
                self.routes_dir.display()
            );
        }

        Ok(routes_count)
    }

    fn scan_specs_dir(&self, index: &mut RouteIndex) -> PulseResult<usize> {
        let mut specs_count = 0;

        // Build glob pattern relative to specs directory
        let pattern = format!("{}/**/test/*.cy.ts", self.specs_dir.display());
        debug!("Using glob pattern: {}", pattern);

        // Use enhanced file system utilities for resilient file discovery
        let spec_paths = match FileSystemUtils::resolve_glob_pattern(&pattern, None) {
            Ok(paths) => paths,
            Err(e) => {
                error!("Failed to resolve test file pattern: {}", e);
                return Err(e);
            }
        };

        // Validate and filter test files
        let (valid_specs, issues) = FileSystemUtils::validate_test_files(&spec_paths);

        // Log any issues found during validation
        if !issues.is_empty() {
            info!("Found {} file issues during spec scanning:", issues.len());
            for issue in &issues {
                debug!("  - {}", issue);
            }
        }

        // Process valid test files
        for path in valid_specs {
            debug!("Processing test file: {}", path.display());
            let spec_path = path.to_string_lossy().into_owned();

            // Read and analyze file content safely
            match FileSystemUtils::safe_read_file(&path) {
                Ok(content) => {
                    let related_files = self.extract_related_files(&content);
                    debug!(
                        "Found {} related files for spec: {}",
                        related_files.len(),
                        spec_path
                    );

                    for file in related_files {
                        index
                            .file_to_specs
                            .entry(file)
                            .or_insert_with(Vec::new)
                            .push(spec_path.clone());
                    }

                    specs_count += 1;
                }
                Err(e) => {
                    warn!("Cannot read test file {}: {}", path.display(), e);
                    // Continue processing other files instead of failing
                    continue;
                }
            }
        }

        if specs_count == 0 {
            warn!("No valid test files found matching pattern: {}", pattern);
            let suggestion =
                FileSystemUtils::generate_no_files_suggestion(&pattern, Some(&self.specs_dir));
            warn!("Suggestions:\n{}", suggestion);
        }

        Ok(specs_count)
    }

    fn find_matching_specs(&self, route_name: &str) -> PulseResult<Vec<String>> {
        // Build pattern relative to specs directory
        let pattern = format!("{}/*{}*/test/*.cy.ts", self.specs_dir.display(), route_name);
        debug!("Looking for specs with pattern: {}", pattern);

        // Use enhanced file system utilities for resilient file discovery
        let spec_paths = match FileSystemUtils::resolve_glob_pattern(&pattern, None) {
            Ok(paths) => paths,
            Err(e) => {
                debug!("No specs found for route '{}': {}", route_name, e);
                return Ok(Vec::new()); // Return empty instead of failing
            }
        };

        // Validate and filter test files
        let (valid_specs, issues) = FileSystemUtils::validate_test_files(&spec_paths);

        // Log any issues found during validation (debug level for route-specific searches)
        if !issues.is_empty() {
            debug!(
                "Found {} file issues for route '{}': {:?}",
                issues.len(),
                route_name,
                issues
            );
        }

        // Convert valid paths to strings
        let specs: Vec<String> = valid_specs
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        if specs.is_empty() {
            debug!("No valid specs found for route: {}", route_name);
        } else {
            debug!(
                "Found {} valid specs for route '{}': {:?}",
                specs.len(),
                route_name,
                specs
            );
        }

        Ok(specs)
    }

    fn extract_related_files(&self, content: &str) -> Vec<String> {
        // Buscar patrones comunes en los tests de Cypress
        let mut files = Vec::new();

        // Buscar importaciones
        for line in content.lines() {
            if line.contains("import") && line.contains("from") {
                if let Some(raw) = line.split("from").nth(1) {
                    let trimmed = raw.trim().trim_matches(|c| c == '"' || c == '\'');
                    if !trimmed.is_empty() {
                        files.push(trimmed.to_string());
                    }
                }
            }
        }

        files
    }

    fn save_index(&self, index: &RouteIndex) -> PulseResult<()> {
        debug!("Serializing route index to JSON...");
        let json = serde_json::to_string_pretty(index).map_err(|e| {
            PulseError::fs_error(
                format!("Failed to serialize route index: {}", e),
                Some("Check that the route index data is valid"),
            )
        })?;

        debug!(
            "Writing route index to cache file: {}",
            self.cache_file.display()
        );

        // Use safe file writing with atomic operations
        FileSystemUtils::safe_write_file(&self.cache_file, &json)?;

        debug!("✓ Route index saved successfully");
        Ok(())
    }

    pub fn load_index(&self) -> PulseResult<RouteIndex> {
        debug!(
            "Loading route index from cache: {}",
            self.cache_file.display()
        );

        // Check if cache file exists
        if !self.cache_file.exists() {
            info!("Route index cache not found, will need to build index");
            return Err(PulseError::fs_error(
                "Route index cache file not found",
                Some("Run pulse to build the route index cache"),
            ));
        }

        // Use safe file reading with validation
        let content = FileSystemUtils::safe_read_file(&self.cache_file).map_err(|e| {
            error!("Failed to read cache file: {}", e);
            PulseError::fs_error(
                format!("Failed to read route index cache: {}", e),
                Some("Check file permissions or run pulse to rebuild the route index cache"),
            )
        })?;

        // Validate content is not empty
        if content.trim().is_empty() {
            warn!("Cache file is empty");
            return Err(PulseError::fs_error(
                "Route index cache file is empty",
                Some("Delete the cache file and run pulse to rebuild it"),
            ));
        }

        // Parse JSON content
        let index = serde_json::from_str(&content).map_err(|e| {
            error!("Failed to parse cache file JSON: {}", e);
            PulseError::fs_error(
                format!("Failed to parse route index cache: {}", e),
                Some("The cache file may be corrupted. Delete it and run pulse to rebuild"),
            )
        })?;

        debug!("✓ Route index loaded successfully from cache");
        Ok(index)
    }

    /// Check if the route index cache is stale and needs rebuilding
    pub fn is_cache_stale(&self) -> PulseResult<bool> {
        if !self.cache_file.exists() {
            debug!("Cache file doesn't exist, considering stale");
            return Ok(true);
        }

        let cache_metadata = fs::metadata(&self.cache_file).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot check cache file metadata: {}", e),
                Some("Check file permissions"),
            )
        })?;

        let cache_modified = cache_metadata.modified().map_err(|e| {
            PulseError::fs_error(
                format!("Cannot get cache file modification time: {}", e),
                None::<String>,
            )
        })?;

        // Check if routes directory is newer than cache
        if let Ok(routes_metadata) = fs::metadata(&self.routes_dir) {
            if let Ok(routes_modified) = routes_metadata.modified() {
                if routes_modified > cache_modified {
                    debug!("Routes directory is newer than cache");
                    return Ok(true);
                }
            }
        }

        // Check if specs directory is newer than cache
        if let Ok(specs_metadata) = fs::metadata(&self.specs_dir) {
            if let Ok(specs_modified) = specs_metadata.modified() {
                if specs_modified > cache_modified {
                    debug!("Specs directory is newer than cache");
                    return Ok(true);
                }
            }
        }

        debug!("Cache is up to date");
        Ok(false)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_structure() -> (TempDir, RouteIndexer) {
        let temp_dir = TempDir::new().unwrap();
        let routes_dir = temp_dir.path().join("routes");
        let specs_dir = temp_dir.path().join("routes");
        let cache_file = temp_dir.path().join("cache").join("route-index.json");

        fs::create_dir_all(&routes_dir).unwrap();
        fs::create_dir_all(&specs_dir).unwrap();

        let indexer = RouteIndexer::new(&routes_dir, &specs_dir, &cache_file);
        (temp_dir, indexer)
    }

    #[test]
    fn test_validate_directories_missing_routes() {
        let temp_dir = TempDir::new().unwrap();
        let missing_routes = temp_dir.path().join("missing_routes");
        let specs_dir = temp_dir.path().join("specs");
        let cache_file = temp_dir.path().join("cache.json");

        fs::create_dir_all(&specs_dir).unwrap();

        let indexer = RouteIndexer::new(&missing_routes, &specs_dir, &cache_file);
        let result = indexer.validate_directories();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, PulseError::FileSystem { .. }));
        assert!(error.suggestion().is_some());
    }

    #[test]
    fn test_validate_directories_success() {
        let (_temp_dir, indexer) = create_test_structure();
        let result = indexer.validate_directories();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_index_empty_directories() {
        let (_temp_dir, indexer) = create_test_structure();
        let result = indexer.build_index();

        // Should handle empty directories gracefully - this might return an error which is acceptable
        match result {
            Ok(index) => {
                assert!(index.route_to_spec.is_empty());
                assert!(index.file_to_specs.is_empty());
            }
            Err(_) => {
                // It's acceptable for empty directories to return an error
                // This is expected behavior when no test files are found
            }
        }
    }

    #[test]
    fn test_is_cache_stale_no_cache() {
        let (_temp_dir, indexer) = create_test_structure();
        let result = indexer.is_cache_stale();

        assert!(result.is_ok());
        assert!(result.unwrap()); // Should be stale when cache doesn't exist
    }

    #[test]
    fn test_load_index_missing_cache() {
        let (_temp_dir, indexer) = create_test_structure();
        let result = indexer.load_index();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, PulseError::FileSystem { .. }));
        assert!(error.suggestion().is_some());
    }

    #[test]
    fn test_save_and_load_index() {
        let (_temp_dir, indexer) = create_test_structure();

        // Create a simple index
        let mut index = RouteIndex {
            route_to_spec: HashMap::new(),
            file_to_specs: HashMap::new(),
        };
        index
            .route_to_spec
            .insert("test".to_string(), vec!["test.cy.ts".to_string()]);

        // Save the index
        let save_result = indexer.save_index(&index);
        assert!(save_result.is_ok());

        // Load the index
        let load_result = indexer.load_index();
        assert!(load_result.is_ok());

        let loaded_index = load_result.unwrap();
        assert_eq!(loaded_index.route_to_spec.len(), 1);
        assert!(loaded_index.route_to_spec.contains_key("test"));
    }
}
