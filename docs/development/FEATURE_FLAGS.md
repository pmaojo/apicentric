# Apicentric Feature Flags

This document explains the feature flags available in Apicentric and how to use them to customize your build.

## Overview

Apicentric uses Cargo feature flags to enable conditional compilation of optional components. This allows you to:
- Reduce binary size by excluding unused features
- Speed up compilation by building only what you need
- Deploy lightweight versions for specific use cases
- Minimize dependencies and attack surface

## Available Features

### Core Features

#### `simulator` (included in default)
The core API simulation engine that powers Apicentric's mock server functionality.

**What it includes:**
- Service definition parsing (YAML/JSON)
- HTTP request routing and response generation
- Template engine for dynamic responses
- State management for stateful mocks

**Binary size impact:** ~2MB
**When to disable:** Never - this is the core functionality

---

#### `contract-testing` (included in default)
Contract testing capabilities for validating API implementations against specifications.

**What it includes:**
- HTTP client for making test requests
- Contract validation logic
- Test report generation

**Dependencies:** `reqwest`, `async-trait`
**Binary size impact:** ~3MB
**When to disable:** If you only need mock servers without testing capabilities

---

### UI Features

#### `tui` (included in default)
Terminal-based user interface using ratatui for interactive service management in the terminal.

**What it includes:**
- Interactive terminal UI for browsing services
- Real-time log viewing in terminal
- Service control (start/stop/restart)
- Configuration management via terminal
- Progress indicators and prompts

**Dependencies:** `ratatui`, `crossterm`, `indicatif`, `console`, `inquire`
**Binary size impact:** ~2MB
**When to disable:** For headless deployments, CI/CD, or when only using WebUI/GUI

**Example:**
```bash
# Build without TUI
cargo build --release --no-default-features --features "simulator,webui"
```

---

#### `gui` (NOT included in default)
Desktop graphical user interface using egui/eframe for native desktop application.

**What it includes:**
- Native desktop GUI application
- Visual service editor
- Interactive configuration
- Cross-platform desktop support (Windows, macOS, Linux)

**Dependencies:** `eframe`, `egui`
**Binary size impact:** ~5MB
**When to enable:** For desktop application usage with native GUI

**Example:**
```bash
# Build with desktop GUI
cargo build --release --features "gui"
```

---

#### `webui` (included in default)
Web-based user interface for browser-based service management.

**What it includes:**
- WebSocket support for real-time updates
- REST API endpoints for UI
- Static file serving for web assets

**Dependencies:** Enables `websockets` feature
**Binary size impact:** ~1MB
**When to disable:** For CLI-only deployments

**Example:**
```bash
# Build without WebUI
cargo build --release --no-default-features --features "simulator,contract-testing,gui"
```

---

#### `tui` (included in default via `gui`)
Low-level terminal UI components.

**What it includes:**
- Terminal rendering (ratatui, crossterm)
- Progress indicators
- Interactive prompts
- Console utilities

**Dependencies:** `ratatui`, `crossterm`, `indicatif`, `console`, `inquire`
**Binary size impact:** ~2MB
**When to disable:** Automatically disabled when `gui` is disabled

---

### Optional Features

#### `p2p` (NOT included in default)
Peer-to-peer collaboration features for sharing service definitions and state.

**What it includes:**
- CRDT-based state synchronization
- libp2p networking stack
- Service definition sharing
- Collaborative editing support

**Dependencies:** `libp2p`, `tokio-stream`, `futures-util`
**Binary size impact:** ~8-10MB
**When to enable:** For collaborative workflows or distributed testing

**Example:**
```bash
# Build with P2P support
cargo build --release --features "p2p"

# Build with only P2P and core features
cargo build --release --no-default-features --features "simulator,p2p"
```

**Why it's optional:**
- Adds significant binary size (~10MB)
- Most users don't need collaboration features
- Reduces dependency count for simpler deployments

---

#### `graphql` (NOT included in default)
GraphQL API support for defining and mocking GraphQL services.

**What it includes:**
- GraphQL schema parsing
- GraphQL query execution
- GraphQL mock response generation

**Dependencies:** `async-graphql`, `async-graphql-parser`
**Binary size impact:** ~4MB
**When to enable:** When working with GraphQL APIs

---

#### `mock-data` (included in default)
Fake data generation for realistic mock responses.

**What it includes:**
- Faker library integration
- Random data generation
- Template helpers for fake data

**Dependencies:** `fake`, `rand`
**Binary size impact:** ~1MB
**When to disable:** If you provide all mock data explicitly

---

#### `database` (included in default)
SQLite database support for persistent state and configuration.

**What it includes:**
- SQLite embedded database
- Configuration persistence
- State storage

**Dependencies:** `rusqlite`
**Binary size impact:** ~2MB
**When to disable:** For stateless deployments

---

#### `file-watch` (included in default)
Automatic reloading when service definition files change.

**What it includes:**
- File system watching
- Automatic service reload
- Hot reloading support

**Dependencies:** `notify`
**Binary size impact:** ~500KB
**When to disable:** For production deployments where files don't change

---

#### `websockets` (included in default via `webui`)
WebSocket support for real-time communication.

**What it includes:**
- WebSocket server
- Real-time log streaming
- Live service updates

**Dependencies:** `tokio-tungstenite`
**Binary size impact:** ~1MB
**When to disable:** Automatically disabled when `webui` is disabled

---

#### `scripting` (included in default)
JavaScript/TypeScript scripting support for advanced request/response manipulation.

**What it includes:**
- Deno runtime integration
- JavaScript execution in hooks
- TypeScript support

**Dependencies:** `deno_core`
**Binary size impact:** ~15MB (largest dependency)
**When to disable:** If you don't use JavaScript hooks

---

## Feature Bundles

### `default`
The standard build with all commonly-used features except P2P.

**Includes:** `gui`, `webui`, `simulator`, `contract-testing`, `tui`, `mock-data`, `database`, `file-watch`, `websockets`, `scripting`

**Use case:** General purpose development and testing

```bash
cargo build --release
```

---

### `full`
Everything including P2P collaboration features.

**Includes:** All features

**Use case:** Maximum functionality for collaborative environments

```bash
cargo build --release --features "full"
```

---

### `minimal`
Bare minimum for API simulation only.

**Includes:** `simulator` only

**Use case:** Lightweight deployments, embedded systems, CI/CD

**Binary size:** ~15-20MB (vs ~50-60MB for default)

```bash
cargo build --release --no-default-features --features "minimal"
```

---

### `cli-tools`
Essential CLI functionality without GUI/WebUI.

**Includes:** `simulator`, `contract-testing`, `tui`

**Use case:** Command-line only environments, servers

```bash
cargo build --release --no-default-features --features "cli-tools"
```

---

### `no-p2p`
Everything except P2P (same as default currently).

**Includes:** All features except `p2p`

**Use case:** Standard deployments without collaboration

```bash
cargo build --release --features "no-p2p"
```

---

## Common Build Configurations

### Lightweight Server Deployment
For running mock servers in production without UI:

```bash
cargo build --release --no-default-features --features "simulator,database"
```

**Size:** ~20MB
**Includes:** Core simulation + persistent storage

---

### CI/CD Testing
For running contract tests in CI pipelines:

```bash
cargo build --release --no-default-features --features "simulator,contract-testing"
```

**Size:** ~25MB
**Includes:** Simulation + testing capabilities

---

### Development Workstation (Terminal UI)
For local development with terminal and web UI:

```bash
# Use default (includes TUI + WebUI)
cargo build --release
```

**Size:** ~50MB
**Includes:** TUI + WebUI for development

---

### Development Workstation (Desktop GUI)
For local development with native desktop GUI:

```bash
cargo build --release --features "gui"
```

**Size:** ~55MB
**Includes:** Desktop GUI + TUI + WebUI

---

### Collaborative Development
For teams working together on API definitions:

```bash
cargo build --release --features "full"
```

**Size:** ~60MB
**Includes:** Everything including P2P collaboration

---

## Feature Dependencies

Some features automatically enable other features:

```
gui → tui → (ratatui, crossterm, indicatif, console, inquire)
webui → websockets → tokio-tungstenite
p2p → (libp2p, tokio-stream, futures-util)
contract-testing → (reqwest, async-trait)
```

## Checking Enabled Features

To see which features are enabled in your build:

```bash
# Check feature flags in Cargo.toml
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "apicentric") | .features'

# Build with verbose output to see enabled features
cargo build --release -vv
```

## Binary Size Comparison

Approximate binary sizes (release builds, stripped):

| Configuration | Size | Build Time |
|--------------|------|------------|
| `minimal` | ~15MB | ~2 min |
| `cli-tools` | ~25MB | ~3 min |
| `default` | ~50MB | ~5 min |
| `full` (with P2P) | ~60MB | ~6 min |

*Note: Actual sizes may vary based on platform and Rust version*

## Performance Impact

Feature flags affect both compile-time and runtime performance:

- **Compile time:** Fewer features = faster builds
- **Binary size:** Fewer features = smaller binaries
- **Runtime:** Disabled features have zero runtime overhead (not compiled)
- **Memory:** Smaller binaries generally use less memory

## Troubleshooting

### Feature Not Available Error

If you see errors like "feature not available" at runtime:

```
Error: P2P feature not available. Enable with --features p2p
```

Rebuild with the required feature:

```bash
cargo build --release --features "p2p"
```

### Dependency Conflicts

If you encounter dependency version conflicts:

1. Check `Cargo.lock` for conflicting versions
2. Try `cargo update` to resolve conflicts
3. Use `cargo tree` to visualize dependency tree

### Build Failures

If build fails with missing dependencies:

1. Ensure you're using a recent Rust version (1.70+)
2. Clear build cache: `cargo clean`
3. Update dependencies: `cargo update`
4. Check that optional dependencies are properly marked

## Contributing

When adding new features:

1. Mark heavy dependencies as `optional = true`
2. Create a feature flag in `[features]` section
3. Document the feature in this file
4. Add feature gates in code: `#[cfg(feature = "your-feature")]`
5. Update tests to cover feature combinations

## See Also

- [Cargo Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture overview
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
