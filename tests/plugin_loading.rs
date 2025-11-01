use mockforge::app::PluginManager;
use std::{path::PathBuf, process::Command};

#[tokio::test]
async fn loads_plugins_from_directory() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status = Command::new("cargo")
        .current_dir(&manifest_dir)
        .args([
            "build",
            "--manifest-path",
            "examples/plugins/logger/Cargo.toml",
        ])
        .status()
        .expect("failed to build plugin");
    assert!(status.success());

    let file_name = format!(
        "{}logger_plugin.{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_EXTENSION
    );
    let mut plugin_path = manifest_dir.join("examples/plugins/logger/target/debug");
    plugin_path.push(&file_name);
    if !plugin_path.exists() {
        plugin_path = manifest_dir.join("target/debug");
        plugin_path.push(&file_name);
    }
    eprintln!("plugin path: {:?}", plugin_path);
    assert!(plugin_path.exists(), "plugin dylib missing");

    let temp_dir = tempfile::tempdir().unwrap();
    let dest = temp_dir.path().join(plugin_path.file_name().unwrap());
    std::fs::copy(&plugin_path, &dest).unwrap();

    let manager = PluginManager::load_from_directory(temp_dir.path())
        .expect("plugin directory should load successfully");
    assert_eq!(manager.plugin_count(), 1);

    use http::{Request, Response};
    let mut req = Request::new(Vec::new());
    let mut res = Response::new(Vec::new());
    manager.on_request(&mut req).await;
    manager.on_response(&mut res).await;
}
