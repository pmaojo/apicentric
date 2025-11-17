<!-- cargo-rdme start -->

# Apicentric

> A powerful CLI tool and API simulator platform for developers who love the terminal
>
> https://apicentric.pelayomaojo.es

## What is Apicentric?

Apicentric is a **Rust-based CLI tool and API simulator platform** that helps developers:

- 🎯 **Mock APIs** with simple YAML configuration
- ✅ **Test API contracts** between services
- 🔄 **Generate code** (TypeScript types, React Query hooks)
- ✨ **MCP** Power your agent with API mocking tools
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

## Real-World Example: E-commerce API

Let's simulate a realistic e-commerce API with dynamic data, request validation, and multiple scenarios.

### 1. Create the Service Definition

Create a file named `ecommerce-api.yaml` with the following content:

```yaml
name: E-commerce API
version: "2.1"
description: Sample e-commerce API with products and orders
server:
  port: 9002
  base_path: /api/v2

fixtures:
  products:
    - id: 101
      name: "Laptop Pro"
      price: 1299.99
      category: "electronics"
      stock: 15
    - id: 102
      name: "Coffee Mug"
      price: 12.50
      category: "home"
      stock: 50

endpoints:
  - method: GET
    path: /products
    description: List products with optional filtering
    parameters:
      - name: category
        in: query
        required: false
        type: string
    responses:
      200:
        content_type: application/json
        body: |
          {
            "products": [
              {{#each fixtures.products}}
              {
                "id": {{id}},
                "name": "{{name}}",
                "price": {{price}},
                "category": "{{category}}",
                "stock": {{stock}}
              }{{#unless @last}},{{/unless}}
              {{/each}}
            ],
            "total": {{fixtures.products.length}},
            "filter": "{{query.category}}"
          }

  - method: POST
    path: /orders
    description: Create a new order
    request_body:
      content_type: application/json
      schema: |
        {
          "customer_id": "number",
          "items": [{"product_id": "number", "quantity": "number"}]
        }
    responses:
      201:
        content_type: application/json
        body: |
          {
            "order_id": {{faker "datatype.number" min=1000 max=9999}},
            "customer_id": {{request.body.customer_id}},
            "items": {{json request.body.items}},
            "total": {{faker "commerce.price"}},
            "status": "pending",
            "created_at": "{{now}}"
          }
      422:
        condition: "{{not request.body.customer_id}}"
        content_type: application/json
        body: |
          {
            "error": "Invalid order",
            "details": ["Customer ID is required"]
          }

  - method: GET
    path: /orders/{id}/status
    description: Get order status
    responses:
      200:
        content_type: application/json
        body: |
          {
            "order_id": {{params.id}},
            "status": "{{#random}}pending,processing,shipped,delivered{{/random}}",
            "updated_at": "{{now}}"
          }

scenarios:
  - name: "holiday_traffic"
    description: "Simulate high traffic during holidays"
    delay_ms: 1500
    response_rate: 0.8

  - name: "maintenance_mode"
    description: "Service under maintenance"
    response:
      status: 503
      headers:
        Retry-After: "3600"
      body: |
        {
          "error": "Service under maintenance",
          "retry_after": "1 hour"
        }
```

### 2. Start the Simulator

Run the following command in your terminal:

```bash
apicentric simulator start --services-dir .
```

Apicentric will start a server on port `9002`.

### 3. Interact with the API

Now you can send requests to your mock API:

**Get all products:**

```bash
curl http://localhost:9002/api/v2/products
```

**Create a new order:**

```bash
curl -X POST http://localhost:9002/api/v2/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_id": 12345,
    "items": [
      {"product_id": 101, "quantity": 1},
      {"product_id": 102, "quantity": 2}
    ]
  }'
```

**Get order status:**

```bash
curl http://localhost:9002/api/v2/orders/5678/status
```

This example demonstrates features like:
- **Fixtures**: Reusable data for your endpoints.
- **Dynamic Responses**: Handlebars templating for realistic data.
- **Request Validation**: Conditional responses based on the request body.
- **Scenarios**: Simulate different API states like high traffic or maintenance.

### 4. Dockerize the Service

Create a portable Docker image for your service:

```bash
apicentric simulator dockerize --services ecommerce-api.yaml --output ./ecommerce-docker
```

This will create a `Dockerfile` and copy the service definition into the `ecommerce-docker` directory. You can then build and run the image:

```bash
cd ecommerce-docker
docker build -t ecommerce-api .
docker run -p 9002:9002 ecommerce-api
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

### Docker

You can use the `dockerize` command to create a self-contained Docker image for your services.

```bash
apicentric simulator dockerize --services <service1>.yaml [<service2>.yaml ...] --output ./my-service-docker
```

This will generate a `Dockerfile` and a `.dockerignore` file in the output directory, along with a `services` directory containing your service definitions.

You can then build and run the image:

```bash
cd my-service-docker
docker build -t my-service .
docker run -p <port>:<port> my-service
```

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
- Import from various formats like OpenAPI, Postman, WireMock, and Mockoon with `apicentric simulator import`.

### GraphQL Mocking

- Define GraphQL mocks with a schema and response templates.
- Create a new GraphQL service from scratch with `apicentric simulator new-graphql <name>`.

### 🐳 Dockerize Services

Package your mock services into self-contained Docker images for easy deployment and sharing.

- Generate a `Dockerfile` for one or more services.
- Exposes all service ports automatically.
- Creates a portable image that can be run anywhere.

### ✅ Contract Testing

Validate that mocks match real APIs:

- Register contracts from specs
- Compare mock vs real responses
- HTML reports with differences
- CI/CD integration

### 🔄 Code Generation & Exporting

Generate client code from service definitions or export to standard formats:

- **Generate TypeScript types**: `apicentric simulator generate-types`
- **Generate React Query hooks**: `apicentric simulator generate-query`
- **Export to OpenAPI**: `apicentric simulator export --format openapi`
- **Export to Postman**: `apicentric simulator export --format postman`

### 🖥️ TUI (Terminal User Interface)

Interactive terminal dashboard for service management:

- Real-time service status
- Live request logs with filtering
- Start/stop services
- Keyboard-driven workflow

### 🤖 AI Integration with MCP (Model Context Protocol)

Apicentric supports the **Model Context Protocol (MCP)**, allowing AI assistants like Claude, ChatGPT, and other MCP-compatible tools to interact with your API simulator programmatically.

#### What is MCP?

MCP is an open protocol that enables AI models to securely access external tools and data sources. With MCP, AI assistants can:

- Create and manage mock API services
- Start/stop services dynamically
- Monitor service logs and status
- Generate service definitions from natural language descriptions

#### Quick MCP Setup

1. **Install with MCP support:**
   ```bash
   cargo install apicentric --features mcp
   # or
   brew install pmaojo/tap/apicentric  # includes MCP
   ```

2. **Configure your AI assistant:**

   For **Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
   ```json
   {
     "mcpServers": {
       "apicentric": {
         "command": "apicentric",
         "args": ["mcp"]
       }
     }
   }
   ```

   For **VS Code** (`.vscode/mcp.json`):
   ```json
   {
     "servers": {
       "apicentric": {
         "type": "stdio",
         "command": "apicentric",
         "args": ["mcp"]
       }
     }
   }
   ```

3. **Start using MCP tools in your AI assistant:**

   ```
   "Create a mock API for a user management system with login and profile endpoints"
   ```

   The AI will use MCP tools to automatically create and start the service!

#### Available MCP Tools

- **`list_services`**: List all available mock services
- **`create_service`**: Create a new service from YAML definition
- **`start_service`**: Start a specific mock service
- **`stop_service`**: Stop a running service
- **`get_service_logs`**: Retrieve logs for a service

#### MCP Example Workflow

**User:** "Create a REST API for managing books with CRUD operations"

**AI Assistant (using MCP tools):**
1. Uses `create_service` to generate a books API YAML
2. Uses `start_service` to launch the API on a port
3. Confirms with `get_service_logs` that it's running
4. Provides curl examples for testing

**Result:** A fully functional mock API ready for testing!

#### MCP Benefits

- **Natural Language API Creation**: Describe your API in plain English
- **Automated Testing Setup**: AI handles service creation and configuration
- **Integrated Development**: Seamless workflow between AI assistance and API development
- **Rapid Prototyping**: Go from idea to working mock API in seconds

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
## Crate Modules

The crate follows hexagonal architecture principles and exposes the following modules:

- `app`: Application bootstrap and command execution.
- `config`: Configuration management for the simulator and tooling.
- `context`: Shared runtime context and dependency wiring.
- `errors`: Custom error types aligned with domain-driven design.
- `logging`: Logging setup and tracing utilities.
- `utils`: Cross-cutting helper functions.
- `validation`: Input validation helpers used across adapters and domain logic.
- `storage`: Persistence adapters for service specifications.
- `ai`: AI-assisted tooling integrations.
- `cloud`: Cloud synchronization utilities.
- `auth`: Authentication helpers for collaborative scenarios.
- `domain`: Core business rules and ports.
- `contract`: Contract testing orchestration.
- `adapters`: Infrastructure adapters that implement ports.
- `simulator`: The API simulator runtime.
- `cli` and `cli_ui`: CLI and text-based UI front-ends.

Refer to the module documentation for deeper implementation details.

<!-- cargo-rdme end -->
