# Developer Experience

Apicentric is designed to be developer-friendly from the inside out. We provide a suite of tools to make contributing and local development a breeze.

## 🛠️ The Supercomplete Makefile

The `Makefile` in the root directory is the central hub for development. It automates complex build steps and ensures environment consistency.

### Core Targets

- `make build`: Compiles both the Rust backend and the Next.js frontend.
- `make dev`: Launches the development environment with hot-reloading for both systems.
- `make clean`: Deep cleans build artifacts, logs, and temporary files.

### Testing & Quality

- `make test`: Executes unit and integration tests.
- `make lint`: Performs static analysis using Clippy and ESLint.
- `make format`: Automatically formats all source code to project standards.

### Specialized tools

- `make tui`: Quickly jumps into the Terminal User Interface for live debugging.
- `make twin`: Starts the IoT Digital Twin simulator with real-world examples.
- `make doctor`: Runs a diagnostic suite to verify your environment setup.

## 🧙 The Management Wizard

If you prefer an interactive workflow, use the **Wizard**:

```bash
make wizard
```

The wizard provides a stylized terminal menu that guides you through:

1. Building the project
2. Launching interfaces (TUI/GUI)
3. Running health checks
4. Executing demo suites
5. Cleaning the workspace

## 📁 Project Structure

For developers, we maintain a clean root directory:

- `logs/`: Centralized build and runtime logs.
- `scripts/`: House for all automation and helper scripts.
- `data/`: Local database storage for simulation state.
- `config/`: Shared configuration for linters and formatters.
- `tests/`: Organized test fixtures and MCP definitions.
