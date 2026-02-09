use std::path::{Path, PathBuf};

/// Resolves a safe path for a service file, preventing directory traversal.
///
/// # Arguments
///
/// * `services_dir` - The directory where services are stored.
/// * `requested_path` - The requested path or filename.
pub fn resolve_safe_service_path(
    services_dir: &str,
    requested_path: &str,
) -> Result<PathBuf, String> {
    let filename = match Path::new(requested_path).file_name() {
        Some(name) => match name.to_str() {
            Some(s) => s,
            None => return Err("Invalid filename encoding".to_string()),
        },
        None => return Err("Invalid path".to_string()),
    };

    Ok(Path::new(services_dir).join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_safe_service_path() {
        let services_dir = "services";

        // Normal case
        let path = resolve_safe_service_path(services_dir, "test.yaml").unwrap();
        #[cfg(not(windows))]
        assert_eq!(path.to_str().unwrap(), "services/test.yaml");
        #[cfg(windows)]
        assert_eq!(path.to_str().unwrap(), "services\\test.yaml");

        // Path traversal attempt
        let path = resolve_safe_service_path(services_dir, "../../etc/passwd").unwrap();
        #[cfg(not(windows))]
        assert_eq!(path.to_str().unwrap(), "services/passwd");
        #[cfg(windows)]
        assert_eq!(path.to_str().unwrap(), "services\\passwd");

        // Subdirectory (should be flattened)
        let path = resolve_safe_service_path(services_dir, "subdir/test.yaml").unwrap();
        #[cfg(not(windows))]
        assert_eq!(path.to_str().unwrap(), "services/test.yaml");
        #[cfg(windows)]
        assert_eq!(path.to_str().unwrap(), "services\\test.yaml");

        // Absolute path (should be flattened)
        #[cfg(not(windows))]
        let path = resolve_safe_service_path(services_dir, "/etc/hosts").unwrap();
        #[cfg(not(windows))]
        #[cfg(not(target_os = "windows"))] // Assuming "not(windows)" covers unix
        if let Some(s) = path.to_str() {
             assert_eq!(s, "services/hosts");
        }

        // Empty path
        assert!(resolve_safe_service_path(services_dir, "").is_err());
    }
}
