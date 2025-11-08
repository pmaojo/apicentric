# Design Document

## Overview

This design document outlines the technical approach for improving the Developer Experience (DX) of Apicentric. The improvements focus on six key areas:

1. **Build Performance Optimization** - Reducing compilation time through dependency analysis and optional features
2. **Documentation Enhancement** - Creating clear, comprehensive English documentation
3. **TUI Enhancement** - Building a rich, interactive terminal interface
4. **Installation Streamlining** - Providing easy installation via multiple channels
5. **CI/CD Pipeline** - Implementing robust GitHub Actions workflows
6. **Open Source Readiness** - Adding community files and best practices

The design maintains Apicentric's existing clean architecture (domain/ports/adapters) while enhancing usability and accessibility.

## Architecture

### Current Architecture Overview

Apicentric follows a layered architecture:

```
┌─────────────────────────────────────────────────┐
│              CLI Layer (clap)                   │
│  Commands: simulator, ai, tui, gui, contract   │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Application Layer (Context)             │
│  ContextBuilder, ExecutionContext, Config       │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│          Domain Layer (Business Logic)          │
│  Contract Testing, Entities, Ports              │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│        Adapter Layer (Infrastructure)           │
│  HTTP Client, Mock Server, Storage, P2P         │
└─────────────────────────────────────────────────┘
```

### Enhanced Architecture

The improvements will add:

1. **Feature Flags System** - Cargo features for optional functionality
2. **Enhanced TUI Module** - Rich terminal interface with real-time updates
3. **CI/CD Infrastructure** - GitHub Actions workflows
4. **Documentation System** - Structured docs with examples

## Components and Interfaces

### 1. Dependency Optimization Module

#### Heavy Dependencies Analysis

Based on analysis, these dependencies are **heavy and optional**:

| Dependency | Size Impact | Usage | Recommendation |
|------------|-------------|-------|----------------|
| `libp2p` (0.56) | **HEAVY** | P2P sharing/collaboration only | Make optional |
| `deno_core` (0.272.0) | **VERY HEAVY** | JavaScript runtime for advanced plugins | Make optional |
| `async-graphql` (7.0) | **HEAVY** | GraphQL mocking only | Make optional |
| `automerge` (0.6) | **HEAVY** | Collaborative editing CRDT | Make optional |
| `hyper/hyper-util` (1.7/0.1) | **MODERATE** | HTTP server | Consider lighter alternative |
| `tokio-tungstenite` (0.21) | **MODERATE** | WebSockets | Make optional |

#### Lightweight Alternatives

For core HTTP functionality, consider:
- **Current**: `hyper` + `hyper-util` (complex, heavy)
- **Alternative**: `axum` (lighter, better DX) or `warp` (minimal)
- **Benefit**: Simpler code, faster builds, better ergonomics

#### Cargo Features Design

```toml
[features]
# Default: minimal, fast-building core
default = ["simulator"]

# Core features (always lightweight)
simulator = []
contract-testing = ["reqwest", "async-trait"]

# Optional heavy features
tui = ["ratatui", "crossterm", "indicatif", "console", "colored", "inquire"]
p2p = ["libp2p", "automerge", "tokio-tungstenite"]
graphql = ["async-graphql", "async-graphql-parser"]
scripting = ["deno_core"]
ai = []

# Convenience bundles
full = ["tui", "p2p", "graphql", "scripting", "ai"]
minimal = ["simulator"]
cli-tools = ["simulator", "contract-testing", "tui"]
```

#### Build Time Estimates

| Feature Set | Dependencies | Estimated Build Time |
|-------------|--------------|---------------------|
| `minimal` | ~30 crates | **< 1 minute** |
| `default` (simulator) | ~40 crates | **< 1.5 minutes** |
| `cli-tools` | ~50 crates | **< 2 minutes** |
| `full` | ~100+ crates | **3-5 minutes** |

#### Implementation Strategy

**Phase 1: Make Heavy Dependencies Optional**

1. Wrap P2P code in `#[cfg(feature = "p2p")]`
2. Wrap GraphQL code in `#[cfg(feature = "graphql")]`
3. Wrap Deno runtime in `#[cfg(feature = "scripting")]`
4. Update CLI to show feature availability

**Phase 2: Evaluate HTTP Server Alternatives**

Research and potentially migrate from `hyper` to `axum`:
- Simpler API
- Better error handling
- Faster compilation
- Built on hyper but with better abstractions

**Phase 3: Document Feature Flags**

```markdown
## Installation Options

### Minimal (fastest build)
```bash
cargo install apicentric --no-default-features --features minimal
```

### CLI Tools (recommended)
```bash
cargo install apicentric --features cli-tools
```

### Full Features
```bash
cargo install apicentric --features full
```
```

### 2. Enhanced TUI Module

#### Component: TuiDashboard

**Purpose**: Provide an interactive terminal interface for managing services and viewing logs.

**Architecture**:
```
TuiDashboard
├── ServiceListPanel (left 25%)
│   ├── Service status indicators
│   ├── Port information
│   └── Selection handling
├── LogViewPanel (center 50%)
│   ├── Real-time log streaming
│   ├── Filtering by method/status
│   └── Scrolling support
└── ActionsPanel (right 25%)
    ├── Keyboard shortcuts
    ├── Service controls
    └── Status information
```

**State Management**:
```rust
#[cfg(feature = "tui")]
pub struct TuiState {
    services: Vec<ServiceStatus>,
    logs: VecDeque<RequestLogEntry>,
    selected_service: Option<String>,
    log_filter: LogFilter,
    scroll_offset: usize,
}

pub struct ServiceStatus {
    name: String,
    port: u16,
    is_running: bool,
    request_count: usize,
    last_request: Option<DateTime<Utc>>,
}

pub struct LogFilter {
    method: Option<String>,
    status: Option<u16>,
    service: Option<String>,
}
```

**Event Loop**:
```rust
#[cfg(feature = "tui")]
pub async fn run_tui(manager: Arc<ApiSimulatorManager>) -> ApicentricResult<()> {
    // Initialize terminal
    let mut terminal = setup_terminal()?;
    
    // Subscribe to log events from simulator
    let mut log_receiver = manager.subscribe_logs();
    
    // Initialize state
    let mut state = TuiState::new();
    
    // Main event loop
    loop {
        // Update service status from manager
        state.update_services(manager.get_status().await).await?;
        
        // Check for new logs (non-blocking)
        while let Ok(log) = log_receiver.try_recv() {
            state.add_log(log);
        }
        
        // Render UI
        terminal.draw(|f| render_ui(f, &state))?;
        
        // Poll for keyboard events (250ms timeout)
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match handle_key_event(key, &mut state, &manager).await? {
                    Action::Quit => break,
                    Action::Continue => {},
                }
            }
        }
    }
    
    restore_terminal(terminal)?;
    Ok(())
}
```

**Key Bindings**:
- `q` / `Ctrl+C`: Quit
- `↑` / `↓`: Navigate services
- `Enter`: Toggle service start/stop
- `f`: Open filter dialog
- `r`: Refresh status
- `c`: Clear logs
- `s`: Save logs to file
- `/`: Search logs
- `Tab`: Switch focus between panels
- `?`: Show help

**UI Layout**:
```
┌─ Services ────────┬─ Request Logs ──────────────────┬─ Actions ─────┐
│ ● api-service     │ 2024-11-08 10:23:45             │ q: Quit       │
│   :9001           │ GET /api/users → 200 OK         │ ↑↓: Navigate  │
│                   │                                  │ ⏎: Start/Stop │
│ ● user-service    │ 2024-11-08 10:23:46             │ f: Filter     │
│   :9002           │ POST /api/login → 201 Created   │ r: Refresh    │
│                   │                                  │ c: Clear      │
│ ○ auth-service    │ 2024-11-08 10:23:47             │ s: Save       │
│   :9003 (stopped) │ GET /api/profile → 200 OK       │ /: Search     │
│                   │                                  │ ?: Help       │
│                   │ [Filtered: GET, 200]            │               │
│                   │ [Scroll: 1-10 of 156]           │ Status: ✓     │
└───────────────────┴─────────────────────────────────┴───────────────┘
```

### 3. Documentation System

#### Structure

```
docs/
├── README.md (main, English)
├── README.es.md (Spanish translation)
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── ARCHITECTURE.md
├── guides/
│   ├── quick-start.md
│   ├── installation.md
│   ├── configuration.md
│   ├── simulator.md
│   ├── contract-testing.md
│   ├── tui.md
│   └── features.md
└── examples/
    ├── basic-api.yaml
    ├── graphql-service.yaml
    └── advanced-scenarios.yaml
```

#### README.md Template

```markdown
# Apicentric

> A powerful CLI tool and API simulator platform for developers who love the terminal

[![CI](badge)](link) [![License: MIT](badge)](link) [![Crates.io](badge)](link)

## What is Apicentric?

Apicentric is a **Rust-based CLI tool and API simulator platform** that helps developers:

- 🎯 **Mock APIs** with simple YAML configuration
- ✅ **Test API contracts** between services
- 🔄 **Generate code** (TypeScript types, React Query hooks)
- 🖥️  **Terminal UI** for visual service management
- 🌐 **P2P collaboration** on service definitions (optional)

Perfect for frontend developers who need backend APIs, teams doing contract testing, or anyone who loves working in the terminal.

## Quick Start

Get up and running in 5 minutes:

```bash
# Install
brew install pmaojo/tap/apicentric

# Create a service
cat > my-api.yaml << EOF
name: my-api
server:
  port: 9000
  base_path: /api
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: '{"message": "Hello, World!"}'
EOF

# Start simulator
apicentric simulator start --services-dir .

# Test it
curl http://localhost:9000/api/hello
```

## Installation

### Homebrew (macOS/Linux)
```bash
brew install pmaojo/tap/apicentric
```

### Install Script (Unix)
```bash
curl -fsSL https://raw.githubusercontent.com/pmaojo/apicentric/main/scripts/install.sh | sh
```

### Cargo
```bash
# Minimal (fastest)
cargo install apicentric --no-default-features --features minimal

# Recommended
cargo install apicentric --features cli-tools

# Full features
cargo install apicentric --features full
```

### Pre-built Binaries
Download from [GitHub Releases](https://github.com/pmaojo/apicentric/releases)

## Features

### 🎯 API Simulator
Define mock APIs in YAML and serve them locally:
- Path parameters and regex matching
- Dynamic templates with Handlebars
- Scenarios for different states
- Request/response logging

### ✅ Contract Testing
Validate that mocks match real APIs:
- Register contracts from specs
- Compare mock vs real responses
- HTML reports with differences
- CI/CD integration

### 🔄 Code Generation
Generate client code from service definitions:
- TypeScript interfaces
- React Query hooks
- OpenAPI specs
- Postman collections

### 🖥️ Terminal UI
Interactive dashboard for service management:
- Real-time service status
- Live request logs with filtering
- Start/stop services
- Keyboard-driven workflow

### 🌐 Advanced Features (Optional)
- **P2P Collaboration**: Share services with team members
- **GraphQL Mocking**: Mock GraphQL APIs with schema
- **JavaScript Plugins**: Extend with custom logic

## Documentation

- [Quick Start Guide](docs/guides/quick-start.md)
- [Installation Guide](docs/guides/installation.md)
- [Simulator Guide](docs/guides/simulator.md)
- [TUI Guide](docs/guides/tui.md)
- [Architecture](docs/ARCHITECTURE.md)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Community

- [GitHub Issues](https://github.com/pmaojo/apicentric/issues)
- [Discussions](https://github.com/pmaojo/apicentric/discussions)
```

### 4. Installation System

#### Component: Release Automation

**GitHub Release Assets**:
- `apicentric-linux-x64.tar.gz`
- `apicentric-macos-x64.tar.gz`
- `apicentric-macos-arm64.tar.gz`
- `apicentric-windows-x64.zip`
- `checksums.txt`

**Installation Scripts**:

```bash
# install.sh (Unix)
#!/bin/bash
set -e

REPO="pmaojo/apicentric"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

echo "Installing Apicentric..."

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Construct download URL
ASSET="apicentric-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

echo "📦 Downloading ${ASSET}..."
curl -fsSL "$URL" -o "/tmp/${ASSET}"

echo "🔍 Verifying checksum..."
curl -fsSL "https://github.com/${REPO}/releases/latest/download/checksums.txt" -o "/tmp/checksums.txt"
cd /tmp && sha256sum -c checksums.txt --ignore-missing

echo "📂 Extracting..."
tar -xzf "/tmp/${ASSET}" -C /tmp

echo "🚀 Installing to ${INSTALL_DIR}..."
sudo mv /tmp/apicentric "${INSTALL_DIR}/apicentric"
sudo chmod +x "${INSTALL_DIR}/apicentric"

echo "✅ Apicentric installed successfully!"
echo "Run 'apicentric --version' to verify."
```

**Homebrew Formula**:
```ruby
class Apicentric < Formula
  desc "CLI tool and API simulator platform for developers"
  homepage "https://github.com/pmaojo/apicentric"
  version "0.1.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-macos-arm64.tar.gz"
      sha256 "..."
    else
      url "https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-macos-x64.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    url "https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-linux-x64.tar.gz"
    sha256 "..."
  end

  def install
    bin.install "apicentric"
  end

  test do
    assert_match "apicentric", shell_output("#{bin}/apicentric --version")
  end
end
```

### 5. CI/CD Pipeline

#### Workflow Architecture

```
GitHub Actions Workflows
├── ci.yml (on push/PR)
│   ├── Format Check (cargo fmt)
│   ├── Lint (cargo clippy)
│   ├── Test (cargo test) - matrix: Linux, macOS, Windows
│   ├── Security Audit (cargo audit)
│   └── Coverage (tarpaulin)
├── release.yml (on tag v*.*.*)
│   ├── Build Binaries (matrix: OS x Arch)
│   ├── Generate Checksums
│   ├── Create GitHub Release
│   └── Upload Assets
└── docs.yml (on push to main)
    └── Deploy Documentation (optional)
```

#### CI Workflow Design

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  format:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      
      - name: Check formatting
        run: cargo fmt --all -- --check

  lint:
    name: Clippy Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
      
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Test Suite
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        features: [minimal, default, full]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}-${{ matrix.features }}
      
      - name: Run tests (minimal)
        if: matrix.features == 'minimal'
        run: cargo test --no-default-features --features minimal
      
      - name: Run tests (default)
        if: matrix.features == 'default'
        run: cargo test
      
      - name: Run tests (full)
        if: matrix.features == 'full'
        run: cargo test --all-features

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features
      
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
```

#### Release Workflow Design

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: false
          prerelease: false

  build:
    name: Build ${{ matrix.target }}
    needs: create-release
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: apicentric-linux-x64
            ext: tar.gz
          
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: apicentric-macos-x64
            ext: tar.gz
          
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: apicentric-macos-arm64
            ext: tar.gz
          
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: apicentric-windows-x64
            ext: zip
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}
      
      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }} --features cli-tools
      
      - name: Package (Unix)
        if: matrix.ext == 'tar.gz'
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../${{ matrix.artifact }}.tar.gz apicentric
      
      - name: Package (Windows)
        if: matrix.ext == 'zip'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../${{ matrix.artifact }}.zip apicentric.exe
      
      - name: Generate checksum
        run: |
          if [ "${{ runner.os }}" = "Windows" ]; then
            certutil -hashfile ${{ matrix.artifact }}.${{ matrix.ext }} SHA256 > ${{ matrix.artifact }}.sha256
          else
            shasum -a 256 ${{ matrix.artifact }}.${{ matrix.ext }} > ${{ matrix.artifact }}.sha256
          fi
        shell: bash
      
      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./${{ matrix.artifact }}.${{ matrix.ext }}
          asset_name: ${{ matrix.artifact }}.${{ matrix.ext }}
          asset_content_type: application/octet-stream
      
      - name: Upload Checksum
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./${{ matrix.artifact }}.sha256
          asset_name: ${{ matrix.artifact }}.sha256
          asset_content_type: text/plain
```

## Data Models

### TUI State Models

```rust
#[cfg(feature = "tui")]
pub struct TuiAppState {
    pub mode: ViewMode,
    pub services: ServiceListState,
    pub logs: LogViewState,
    pub input: InputState,
    pub manager: Arc<ApiSimulatorManager>,
}

#[cfg(feature = "tui")]
pub enum ViewMode {
    Normal,
    FilterDialog,
    SearchDialog,
    HelpDialog,
}

pub struct ServiceListState {
    pub items: Vec<ServiceStatus>,
    pub selected: usize,
}

pub struct LogViewState {
    pub entries: VecDeque<RequestLogEntry>,
    pub filter: LogFilter,
    pub scroll: usize,
    pub max_entries: usize, // Default: 1000
}

pub struct InputState {
    pub buffer: String,
    pub cursor: usize,
}
```

## Error Handling

The existing `ApicentricError` enum provides excellent error handling. We'll extend it minimally:

```rust
impl ApicentricError {
    #[cfg(feature = "tui")]
    pub fn tui_error(message: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::Runtime {
            message: format!("TUI error: {}", message.into()),
            suggestion: suggestion.map(|s| s.into()),
        }
    }
}
```

All errors use `ErrorFormatter` for user-friendly output:
```
❌ Configuration error: Invalid simulator port range
💡 Suggestion: Port range must be between 1024 and 65535
🔍 Field: simulator.port_range
```

## Testing Strategy

### Unit Tests
- Dependency analysis logic
- TUI state management
- Feature flag compilation
- Error formatting

### Integration Tests
- TUI with real simulator
- Installation scripts
- CLI commands with different features

### CI Tests
- Format: `cargo fmt --all -- --check`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Test: `cargo test` on Linux, macOS, Windows with minimal/default/full features
- Audit: `cargo audit`
- Coverage: `cargo tarpaulin --out Xml --all-features`

## Performance Considerations

### Build Time Optimization

**Measurement Baseline**:
```bash
# Measure current build time
time cargo clean && cargo build --release

# Measure with features
time cargo build --release --no-default-features --features minimal
time cargo build --release --features cli-tools
time cargo build --release --features full
```

**Target Build Times**:
- `minimal`: < 1 minute
- `default`: < 1.5 minutes
- `cli-tools`: < 2 minutes
- `full`: < 5 minutes

**Optimization Strategies**:
1. Feature flags for heavy dependencies
2. CI dependency caching
3. Parallel compilation (already enabled by Cargo)
4. Consider lighter HTTP server (axum vs hyper)

### TUI Performance

**Target**: < 500ms UI update latency

**Strategies**:
- Efficient rendering (only changed regions)
- Async updates with tokio
- Bounded log buffer (max 1000 entries)
- Debounced rapid updates

### CI Performance

**Target**: < 10 minutes for full pipeline

**Strategies**:
- Dependency caching with `Swatinem/rust-cache`
- Parallel jobs (format, lint, test)
- Matrix strategy for multi-platform tests
- Incremental builds where possible

## Security Considerations

### Dependency Security
- `cargo audit` on every PR
- Dependabot for automatic updates
- Review heavy dependencies for vulnerabilities

### Installation Security
- SHA256 checksums for all binaries
- HTTPS-only downloads
- Consider GPG signing for releases

### Runtime Security
- Input validation (already implemented)
- Path traversal prevention
- Deno sandbox for scripts (already implemented)

## Migration Strategy

### Backward Compatibility
All changes maintain backward compatibility:
- Existing commands work unchanged
- Configuration files remain compatible
- Feature flags are additive only
- Default features provide current functionality

### Feature Flag Migration
Users can opt into lighter builds:
```bash
# Before (builds everything)
cargo install apicentric

# After (same behavior, explicit)
cargo install apicentric --features cli-tools

# New option (minimal, fast)
cargo install apicentric --no-default-features --features minimal
```

## Success Metrics

### Build Performance
- [ ] Minimal build < 1 minute
- [ ] Default build < 1.5 minutes
- [ ] CLI tools build < 2 minutes
- [ ] CI pipeline < 10 minutes

### Documentation
- [ ] README clarity (user feedback)
- [ ] Quick start < 5 minutes
- [ ] All commands documented

### TUI
- [ ] UI update < 500ms
- [ ] All keyboard shortcuts work
- [ ] Real-time log streaming
- [ ] Service management functional

### Installation
- [ ] Install time < 30 seconds
- [ ] Works on all platforms
- [ ] Checksums verify

### CI/CD
- [ ] All checks pass
- [ ] Releases automated
- [ ] Code coverage > 70%
- [ ] Zero critical vulnerabilities

### Community
- [ ] GitHub stars growth
- [ ] External contributors
- [ ] Active discussions
