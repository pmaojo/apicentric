pub mod generate_docs;
pub mod mock_api_service;
pub mod setup_npm;

use std::{fs, path::Path};
use http::{Request, Response};
use libloading::{Library, Symbol};
use crate::domain::ports::plugin::Plugin;

/// Manages dynamically loaded plugins and executes their hooks.
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    _libraries: Vec<Library>,
}

impl PluginManager {
    /// Create an empty manager with no plugins loaded.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            _libraries: Vec::new(),
        }
    }

    /// Load all plugins from the specified directory.
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Self {
        let mut manager = Self::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().and_then(|s| s.to_str())
                        == Some(std::env::consts::DLL_EXTENSION)
                {
                    unsafe {
                        if let Ok(lib) = Library::new(&path) {
                            type PluginCreate = fn() -> Box<dyn Plugin>;
                            let constructor: Symbol<PluginCreate> =
                                lib.get(b"create_plugin").expect("create_plugin symbol");
                            manager.plugins.push(constructor());
                            manager._libraries.push(lib);
                        }
                    }
                }
            }
        }
        manager
    }

    /// Register a plugin instance manually.
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Number of loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Execute request hooks for all plugins.
    pub async fn on_request(&self, req: &mut Request<Vec<u8>>) {
        for plugin in &self.plugins {
            plugin.on_request(req).await;
        }
    }

    /// Execute response hooks for all plugins.
    pub async fn on_response(&self, res: &mut Response<Vec<u8>>) {
        for plugin in &self.plugins {
            plugin.on_response(res).await;
        }
    }
}
