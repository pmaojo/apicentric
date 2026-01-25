# Plugin Development Guide

> **Note:** The plugin system in Apicentric is currently **experimental** and intended for advanced users building custom integrations programmatically. Plugins are not yet supported via the standard CLI configuration or `apicentric.json`.

Plugins allow you to extend Apicentric's functionality by intercepting requests and responses, adding custom logging, or modifying behavior dynamically. Apicentric plugins are dynamic libraries (`.dll`, `.so`, `.dylib`) written in Rust that implement the `Plugin` trait.

## What is a Plugin?

A plugin is a shared library that:
1. Implements the `apicentric::domain::ports::plugin::Plugin` trait.
2. Exports a C-compatible function named `create_plugin` that returns a boxed instance of the plugin.

## Creating a Plugin

To create a plugin, you need to set up a new Rust library project with `crate-type = ["cdylib"]`.

### 1. Cargo.toml

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
async-trait = "0.1"
log = "0.4"
http = "1"
# Depend on apicentric to access the Plugin trait
apicentric = { version = "0.4" }
```

### 2. Implementation (`src/lib.rs`)

```rust
use async_trait::async_trait;
use http::{Request, Response};
use log::info;
use apicentric::domain::ports::plugin::Plugin;

struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn on_request(&self, request: &mut Request<Vec<u8>>) {
        info!("Intercepted request: {} {}", request.method(), request.uri());
        // You can modify the request headers or body here
    }

    async fn on_response(&self, response: &mut Response<Vec<u8>>) {
        info!("Intercepted response: {}", response.status());
        // You can modify the response here
    }
}

// Required export to instantiate the plugin
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin)
}
```

### 3. Build

Build the plugin as a dynamic library:

```bash
cargo build --release
```

The output file will be in `target/release/`:
- Linux: `libmy_plugin.so`
- macOS: `libmy_plugin.dylib`
- Windows: `my_plugin.dll`

## Using Plugins

Currently, plugins must be loaded programmatically using the `PluginManager` within a custom Rust application that uses `apicentric` as a library.

```rust
use apicentric::app::PluginManager;

// Load all plugins from a directory
let manager = PluginManager::load_from_directory("path/to/plugins")?;

// The manager is typically integrated into the request processing pipeline
// (Note: Integration into the main simulator pipeline is a work in progress)
```

## Example

A complete example is available in the `examples/plugins/logger` directory of the repository.

## Limitations

- **ABI Stability:** Rust does not have a stable ABI. Plugins must be compiled with the **exact same rustc version** as the host application.
- **Allocator Mismatch (Windows):** On Windows, plugins must share the memory allocator with the host to safely pass heap-allocated objects (like `Box<dyn Future>`). This currently limits the ability to safely execute complex plugins on Windows without advanced setup.
