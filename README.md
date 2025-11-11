# Apicentric

> A powerful CLI tool and API simulator platform for developers who love the terminal

## What is Apicentric?

Apicentric is a **Rust-based CLI tool and API simulator platform** that helps developers:

- 🎯 **Mock APIs** with simple YAML configuration
- ✅ **Test API contracts** between services
- 🔄 **Generate code** (TypeScript types, React Query hooks)
- 🖥️ **TUI (Terminal User Interface)** for visual service management
- 🌐 **P2P collaboration** on service definitions (optional)

Perfect for frontend developers who need backend APIs, teams doing contract testing, or anyone who loves working in the terminal.

## Core Concepts

Apicentric is built around a few core concepts:

- **Service Definition**: A YAML file that defines a mock API, including its endpoints, responses, and scenarios.
- **Simulator**: A local server that serves the mock APIs defined in your service definitions.
- **Contract Testing**: A feature that allows you to validate that your mock APIs match the real APIs they are mocking.
- **Code Generation**: A feature that allows you to generate client code from your service definitions.
- **TUI**: A terminal user interface that provides a visual way to manage your services.

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

Apicentric provides multiple installation methods to suit your workflow. Choose the one that works best for you.

### Homebrew (macOS/Linux) - Recommended

The easiest way to install on macOS and Linux:

```bash
brew install pmaojo/tap/apicentric
```

**Verify installation:**

```bash
apicentric --version
```

**Update to latest version:**

```bash
brew upgrade apicentric
```

### Install Script (Unix)

Quick installation script for Linux and macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/pmaojo/apicentric/main/scripts/install.sh | sh
```

This script will:
- Detect your platform and architecture automatically
- Download the appropriate binary
- Verify checksums for security
- Install to `/usr/local/bin` (requires sudo)

**Custom installation directory:**

```bash
INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/pmaojo/apicentric/main/scripts/install.sh | sh
```

**Verify installation:**

```bash
apicentric --version
```

### Windows PowerShell

For Windows users, use the PowerShell installation script:

```powershell
irm https://raw.githubusercontent.com/pmaojo/apicentric/main/scripts/install.ps1 | iex
```

This script will:
- Download the Windows x64 binary
- Verify checksums
- Extract to `%USERPROFILE%\.apicentric\bin`
- Add to PATH (restart terminal after installation)

**Verify installation:**

```powershell
apicentric --version
```

### Cargo (Build from Source)

If you have Rust installed, you can build from source with custom features:

**Minimal build (fastest, ~1 minute):**

```bash
cargo install apicentric --no-default-features --features minimal
```

Includes: Core simulator only

**CLI Tools build (recommended, ~2 minutes):**

```bash
cargo install apicentric --features cli-tools
```

Includes: Simulator, contract testing, and TUI

**Full build (all features, ~3-5 minutes):**

```bash
cargo install apicentric --features full
```

Includes: All features (TUI, P2P, GraphQL, scripting, AI)

**Default build:**

```bash
cargo install apicentric
```

Includes: Simulator and contract testing

**Verify installation:**

```bash
apicentric --version
```

### Pre-built Binaries

Download pre-built binaries for your platform from [GitHub Releases](https://github.com/pmaojo/apicentric/releases).

**Available platforms:**
- Linux x64 (`apicentric-linux-x64.tar.gz`)
- macOS x64 (`apicentric-macos-x64.tar.gz`)
- macOS ARM64 (`apicentric-macos-arm64.tar.gz`)
- Windows x64 (`apicentric-windows-x64.zip`)

**Manual installation (Linux/macOS):**

```bash
# Download the appropriate archive
curl -LO https://github.com/pmaojo/apicentric/releases/latest/download/apicentric-linux-x64.tar.gz

# Verify checksum (optional but recommended)
curl -LO https://github.com/pmaojo/apicentric/releases/latest/download/checksums.txt
sha256sum -c checksums.txt --ignore-missing

# Extract
tar -xzf apicentric-linux-x64.tar.gz

# Move to PATH
sudo mv apicentric /usr/local/bin/

# Make executable
sudo chmod +x /usr/local/bin/apicentric
```

**Manual installation (Windows):**

1. Download `apicentric-windows-x64.zip` from releases
2. Extract the archive
3. Move `apicentric.exe` to a directory in your PATH
4. Or add the directory to your PATH environment variable

**Verify installation:**

```bash
apicentric --version
```

### Docker (Coming Soon)

Docker images will be available soon for containerized deployments.

## Verification

After installation, verify that Apicentric is working correctly:

```bash
# Check version
apicentric --version

# View help
apicentric --help

# List available commands
apicentric simulator --help
```

Expected output should show version information and available commands.

## Troubleshooting

### Command not found

**Issue:** `apicentric: command not found` after installation

**Solutions:**

- **Homebrew:** Ensure Homebrew's bin directory is in your PATH:
  ```bash
  echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
  source ~/.bashrc
  ```

- **Install script:** Verify `/usr/local/bin` is in your PATH:
  ```bash
  echo $PATH | grep -q "/usr/local/bin" && echo "✓ In PATH" || echo "✗ Not in PATH"
  ```

- **Windows:** Restart your terminal or PowerShell after installation to refresh PATH

- **Cargo:** Ensure `~/.cargo/bin` is in your PATH:
  ```bash
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
  source ~/.bashrc
  ```

### Permission denied

**Issue:** Permission errors during installation

**Solutions:**

- **Unix install script:** The script requires sudo for `/usr/local/bin`. Use custom directory:
  ```bash
  INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/pmaojo/apicentric/main/scripts/install.sh | sh
  ```
  Then add to PATH:
  ```bash
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
  source ~/.bashrc
  ```

- **Manual installation:** Use `sudo` when moving to system directories:
  ```bash
  sudo mv apicentric /usr/local/bin/
  sudo chmod +x /usr/local/bin/apicentric
  ```

### Checksum verification failed

**Issue:** Checksum mismatch during installation

**Solutions:**

- Download may be corrupted. Delete and try again:
  ```bash
  rm apicentric-*.tar.gz
  curl -LO https://github.com/pmaojo/apicentric/releases/latest/download/apicentric-linux-x64.tar.gz
  ```

- Verify you're downloading from the official repository
- Check your internet connection

### Cargo build fails

**Issue:** Compilation errors when building from source

**Solutions:**

- **Update Rust:** Ensure you have the latest stable Rust:
  ```bash
  rustup update stable
  ```

- **Missing dependencies:** Install required system dependencies:
  - **Ubuntu/Debian:**
    ```bash
    sudo apt-get update
    sudo apt-get install build-essential pkg-config libssl-dev
    ```
  - **macOS:**
    ```bash
    xcode-select --install
    ```
  - **Windows:** Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

- **Try minimal build:** If full build fails, try minimal:
  ```bash
  cargo install apicentric --no-default-features --features minimal
  ```

### Feature not available

**Issue:** Command shows "Feature not available in this build"

**Solutions:**

- You installed a minimal build. Reinstall with desired features:
  ```bash
  cargo install apicentric --features cli-tools --force
  ```

- Or install full version:
  ```bash
  brew reinstall apicentric  # Homebrew includes cli-tools features
  ```

### macOS security warning

**Issue:** "apicentric cannot be opened because it is from an unidentified developer"

**Solutions:**

- **Option 1:** Use Homebrew installation (recommended):
  ```bash
  brew install pmaojo/tap/apicentric
  ```

- **Option 2:** Allow the binary manually:
  ```bash
  xattr -d com.apple.quarantine /usr/local/bin/apicentric
  ```

- **Option 3:** Build from source with Cargo:
  ```bash
  cargo install apicentric --features cli-tools
  ```

### Still having issues?

If you're still experiencing problems:

1. Check [GitHub Issues](https://github.com/pmaojo/apicentric/issues) for similar problems
2. Create a new issue with:
   - Your operating system and version
   - Installation method used
   - Complete error message
   - Output of `apicentric --version` (if available)
3. Join our [Discussions](https://github.com/pmaojo/apicentric/discussions) for community support

## Features

### 🎯 API Simulator

Define mock APIs in YAML and serve them locally:

- Path parameters and regex matching
- Dynamic templates with Handlebars
- Scenarios for different states
- Request/response logging
- Request recording proxy and auto-generated endpoints via `record_unknown`
- Imports OpenAPI 2.0/3.x specs, preferring documented examples and generating JSON bodies from schemas when necessary

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

### 🖥️ TUI (Terminal User Interface)

Interactive terminal dashboard for service management:

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
- [Request Recording Guide](docs/guides/request-recording.md)
- [Installation Guide](docs/guides/installation.md)
- [Simulator Guide](docs/guides/simulator.md)
- [TUI Guide](docs/guides/tui.md)
- [Architecture](ARCHITECTURE.md)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Community

- [GitHub Issues](https://github.com/pmaojo/apicentric/issues)
- [Discussions](https://github.com/pmaojo/apicentric/discussions)
