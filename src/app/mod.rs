//! Manages dynamically loaded plugins and executes their hooks.
//!
//! This module provides a `PluginManager` that can be used to load plugins
//! from a directory, register them, and execute their hooks.

pub mod mock_api_service;
pub mod setup_npm;

use crate::domain::ports::plugin::Plugin;
use crate::errors::{ApicentricError, ApicentricResult};
use http::{Request, Response};
use libloading::{Library, Symbol};
use std::{fs, path::Path};

/// Manages dynamically loaded plugins and executes their hooks.
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    _libraries: Vec<Library>,
}

impl PluginManager {
    /// Creates an empty `PluginManager` with no plugins loaded.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            _libraries: Vec::new(),
        }
    }

    /// Loads all plugins from the specified directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory to load plugins from.
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> ApicentricResult<Self> {
        let dir_path = dir.as_ref();
        let entries = fs::read_dir(dir_path).map_err(|err| {
            ApicentricError::fs_error(
                format!(
                    "Failed to read plugin directory '{}': {}",
                    dir_path.display(),
                    err
                ),
                Some("Ensure the plugin directory exists and is accessible."),
            )
        })?;

        let mut manager = Self::new();
        let mut failures: Vec<String> = Vec::new();

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(err) => {
                    failures.push(format!(
                        "Failed to read an entry in '{}': {}",
                        dir_path.display(),
                        err
                    ));
                    continue;
                }
            };

            let path = entry.path();
            if !Self::is_dynamic_library(&path) {
                continue;
            }

            match unsafe { Self::instantiate_plugin(&path) } {
                Ok((plugin, library)) => {
                    manager.plugins.push(plugin);
                    manager._libraries.push(library);
                }
                Err(err) => failures.push(err.to_string()),
            }
        }

        if failures.is_empty() {
            Ok(manager)
        } else {
            let mut message = String::from("Failed to load one or more plugins:");
            for failure in failures {
                message.push_str("\n  - ");
                message.push_str(&failure);
            }

            Err(ApicentricError::runtime_error(
                message,
                Some(
                    "Ensure each plugin exports a `create_plugin` symbol and is built for the current platform.",
                ),
            ))
        }
    }

    fn is_dynamic_library(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => ext.eq_ignore_ascii_case(std::env::consts::DLL_EXTENSION),
            None => false,
        }
    }

    unsafe fn instantiate_plugin(path: &Path) -> ApicentricResult<(Box<dyn Plugin>, Library)> {
        let library = Library::new(path).map_err(|err| {
            ApicentricError::runtime_error(
                format!(
                    "Failed to open plugin library '{}': {}",
                    path.display(),
                    err
                ),
                Some("Ensure the plugin file is a valid dynamic library built for this platform."),
            )
        })?;

        type PluginCreate = fn() -> Box<dyn Plugin>;
        let constructor: Symbol<PluginCreate> = library.get(b"create_plugin").map_err(|err| {
            ApicentricError::runtime_error(
                format!(
                    "Plugin library '{}' is missing the required `create_plugin` symbol: {}",
                    path.display(),
                    err
                ),
                Some(
                    "Ensure the plugin exports a `create_plugin` function with signature `fn() -> Box<dyn Plugin>`.",
                ),
            )
        })?;

        Ok((constructor(), library))
    }

    /// Registers a plugin instance manually.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The plugin to register.
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Returns the number of loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Executes the `on_request` hook for all plugins.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request.
    pub async fn on_request(&self, req: &mut Request<Vec<u8>>) {
        for plugin in &self.plugins {
            plugin.on_request(req).await;
        }
    }

    /// Executes the `on_response` hook for all plugins.
    ///
    /// # Arguments
    ///
    /// * `res` - The HTTP response.
    pub async fn on_response(&self, res: &mut Response<Vec<u8>>) {
        for plugin in &self.plugins {
            plugin.on_response(res).await;
        }
    }
}
