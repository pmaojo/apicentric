#[cfg(unix)]
#[cfg(test)]
mod tests {
    use apicentric::simulator::config::validation::{ConfigFileLoader, ConfigRepository};
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    #[test]
    fn test_symlink_traversal() {
        // Setup directory structure
        let dir = tempdir().unwrap();
        let base_path = dir.path();

        let external_dir = base_path.join("external");
        fs::create_dir(&external_dir).unwrap();

        let secret_file = external_dir.join("secret.yaml");
        fs::write(&secret_file, "name: secret").unwrap();

        let services_dir = base_path.join("services");
        fs::create_dir(&services_dir).unwrap();

        let symlink_path = services_dir.join("link");
        symlink("../external", &symlink_path).unwrap();

        // Use ConfigFileLoader to list files
        let loader = ConfigFileLoader::new(services_dir.clone());
        let files = loader.list_service_files().unwrap();

        // Assert that the secret file IS NOT found (confirming fix)
        // If the fix works, the loader skips the symlink and does not find secret.yaml
        let found = files
            .iter()
            .any(|p| p.file_name().unwrap() == "secret.yaml");

        // We expect this to be false (fix confirmed)
        assert!(
            !found,
            "Vulnerability persisted: secret.yaml was found via symlink"
        );
    }
}
