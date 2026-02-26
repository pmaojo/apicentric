#![cfg(feature = "webui")]

use apicentric::simulator::marketplace::install_template;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_marketplace_path_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let services_dir = temp_dir.path().join("services");
    std::fs::create_dir(&services_dir).unwrap();

    // Use a non-network test first if possible, but install_template is network-bound.
    // Instead, we will simulate the behavior manually to verify the vulnerability logic
    // without hitting the network, OR we assume the network call fails and just verify the path logic if we can mock it.
    // Since we cannot easily mock the network call in this integration test without setting up a full mock server,
    // we will rely on the code analysis:

    // Code analysis:
    // let file_name = format!("{}.yaml", definition.name.to_lowercase().replace(' ', "-"));
    // let file_path = output_dir.join(&file_name);

    // If definition.name (or name_override) is "../pwned",
    // file_name becomes "../pwned.yaml".
    // file_path becomes output_dir/../pwned.yaml -> parent/pwned.yaml.

    // This IS a vulnerability.

    // To confirm it without network, we can verify if the name sanitization is sufficient.
    // It only replaces spaces with dashes. It does NOT strip '..' or '/'.

    let unsafe_name = "../pwned";
    let file_name = format!("{}.yaml", unsafe_name.to_lowercase().replace(' ', "-"));

    assert_eq!(file_name, "../pwned.yaml");

    let file_path = services_dir.join(&file_name);
    // This resolves to temp_dir/pwned.yaml, outside services_dir.

    // The fix should sanitize the name using Path::file_name() or a regex to remove dangerous chars.
}
