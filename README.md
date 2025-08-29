```
╔══════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                      ║
║  ██████╗ ██╗   ██╗██╗     ███████╗███████╗    ████████╗███████╗███████╗████████╗    ║
║  ██╔══██╗██║   ██║██║     ██╔════╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝    ║
║  ██████╔╝██║   ██║██║     ███████╗█████╗         ██║   █████╗  ███████╗   ██║       ║
║  ██╔═══╝ ██║   ██║██║     ╚════██║██╔══╝         ██║   ██╔══╝  ╚════██║   ██║       ║
║  ██║     ╚██████╔╝███████╗███████║███████╗       ██║   ███████╗███████║   ██║       ║
║  ╚═╝      ╚═════╝ ╚══════╝╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝       ║
║                                                                                      ║
║                    ⚡ INTELLIGENT RUST-POWERED TEST RUNNER ⚡                       ║
║                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════╝
```

```
┌─ SYSTEM STATUS ──────────────────────────────────────────────────────────────────────┐
│                                                                                      │
│  🎯 MISSION: High-performance intelligent test execution for modern web apps         │
│  🔧 ENGINE:  Rust-powered with Cypress integration                                   │
│  📊 METRICS: Real-time monitoring with Prometheus, Sentry & Allure                   │
│  🚀 SPEED:   Parallel execution with smart impact analysis                           │
│                                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

## ⚡ CORE FEATURES

```
╭─ SMART EXECUTION ENGINE ─────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🧠 IMPACT ANALYSIS     │ Maps code changes → relevant test files automatically     │
│  ⚡ PARALLEL WORKERS     │ Configurable worker pools for maximum throughput         │
│  🔄 RETRY LOGIC         │ Intelligent retry mechanism for flaky test handling       │
│  👁️  WATCH MODE          │ Real-time file monitoring with debounced execution       │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ SERVER MANAGEMENT ──────────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🚀 AUTO-START          │ Automatically launch development servers                  │
│  💓 HEALTH CHECKS       │ Configurable server health monitoring                     │
│  🔧 PROCESS CONTROL     │ Clean server lifecycle management                         │
│  ⏱️  TIMEOUT HANDLING    │ Smart timeout and retry configurations                    │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ REPORTING & METRICS ────────────────────────────────────────────────────────────────╮
│                                                                                      │
│  📋 JUNIT REPORTS       │ Standard XML test reports with consolidation              │
│  🎭 ALLURE INTEGRATION  │ Rich test reporting with screenshots and logs             │
│  📊 PROMETHEUS METRICS  │ Performance and reliability metrics                       │
│  🔍 SENTRY MONITORING   │ Error tracking and performance monitoring                 │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ DEVELOPER EXPERIENCE ───────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🎛️  EXECUTION MODES     │ CI, Development, and Debug modes                          │
│  🏃 DRY RUN             │ Preview test execution without running                     │
│  📦 NPM INTEGRATION     │ Seamless package.json script setup                        │
│  🔍 VERBOSE LOGGING     │ Detailed debugging and execution information              │
│  🦐 MOCK API SIMULATOR  │ YAML-driven local API (endpoints, delays, scenarios)       │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

## 🛠️ INSTALLATION

```

┌─ SYSTEM REQUIREMENTS ───────────────────────────────────────────────────────────────┐
│ │
│ ⚙️ RUST │ v1.70+ │ Core runtime engine │
│ 📦 NODE.JS │ v16+ │ Cypress integration layer │
│ 🌐 NPM │ Latest │ Package management │
│ 🧪 CYPRESS │ v12+ │ Test execution framework │
│ │
└──────────────────────────────────────────────────────────────────────────────────────┘

```

```

╭─ BUILD PROCESS ──────────────────────────────────────────────────────────────────────╮
│ │
│ $ cd utils/pulse │
│ $ cargo build --release │
│ │
│ ✅ Binary location: utils/pulse/target/release/pulse │
│ │
╰──────────────────────────────────────────────────────────────────────────────────────╯

````

## Quick Start

### 1. Initialize Configuration
```bash
# Create a default pulse.json configuration
pulse init

# Or initialize with force overwrite
pulse init --force
````

### 2. Configure Your Project

Edit the generated `pulse.json` file:

```json
{
  "cypress_config_path": "cypress.config.ts",
  "base_url": "http://localhost:5173",
  "specs_pattern": "app/routes/**/test/*.cy.ts",
  "routes_dir": "app/routes",
  "specs_dir": "app/routes",
  "reports_dir": "cypress/reports",
  "index_cache_path": ".pulse/route-index.json",
  "default_timeout": 30000,
  "server": {
    "auto_start": true,
    "start_command": "npm run dev",
    "startup_timeout_ms": 30000,
    "health_check_retries": 5
  },
  "execution": {
    "mode": "development",
    "continue_on_failure": true,
    "dry_run": false,
    "verbose": false
  }
}
```

### 3. Set Up NPM Scripts

```bash
# Add pulse scripts to package.json
pulse setup-npm

# Or just show instructions
pulse setup-npm --instructions-only
```

### 4. Run Tests

```bash
# Watch mode - runs impacted tests on file changes
pulse watch

# Run all tests once
pulse run

# Run with custom configuration
pulse --config custom-pulse.json run --workers 8 --retries 2
```

## Usage

### Command Line Interface

```bash
pulse [OPTIONS] <COMMAND>

Commands:
  watch       Watch for changes and run impacted tests
  run         Run all tests once
  setup-npm   Setup npm scripts for pulse integration
  docs        Generate TypeScript documentation
  mock-api    Serve a YAML-defined mock API server

Options:
  -c, --config <CONFIG>    Path to pulse.json config file [default: pulse.json]
      --mode <MODE>        Execution mode [possible values: ci, development, debug]
      --dry-run           Enable dry-run mode (show what would be executed)
  -v, --verbose           Enable verbose output
  -h, --help              Print help

Mock API usage:
  pulse mock-api --spec pulse-mock.yaml
  pulse mock-api --spec pulse-mock.yaml --validate
```

### Watch Mode

Monitor file changes and automatically run impacted tests:

```bash
# Basic watch mode
pulse watch

# Custom configuration
pulse watch --workers 6 --retries 1 --debounce-ms 2000

# Debug mode with verbose output
pulse --mode debug --verbose watch
```

### Run All Tests

Execute the complete test suite:

```bash
# Run all tests
pulse run

# Run with 8 parallel workers
pulse run --workers 8

# CI mode (headless, no server management)
pulse --mode ci run
```

### NPM Integration

Set up convenient npm scripts that automatically run from the project root:

```bash
# Setup scripts in package.json
pulse setup-npm

# Force overwrite existing scripts
pulse setup-npm --force

# Test the npm integration
pulse setup-npm --test

# Show usage examples
pulse setup-npm --examples
```

After setup, you can use these npm scripts from anywhere in your project:

```bash
# These automatically run from the project root directory
npm run pulse -- run
npm run pulse:watch
```

## 🦐 Mock API Simulator (Experimental)

YAML data-driven local API para simular servicios y trabajar offline.

### Ejemplo `pulse-mock.yaml`

```yaml
name: remora-sim
port: 7070
base_path: /api
endpoints:
  - method: GET
    path: /permisos
    status: 200
    response:
      permisos:
        - id: 1
          nombre: VER_HOME
        - id: 2
          nombre: VER_ADMIN
  - method: POST
    path: /login
    status: 200
    delay_ms: 300
    headers:
      X-Auth: mock-token
    response:
      token: abc123
      usuario:
        id: 42
        nombre: demo
  - method: GET
    path: /usuarios/{id}
    status: 200
    response:
      id: 42
      nombre: Usuario Parametrico
```

### Comandos

```bash
pulse mock-api --spec pulse-mock.yaml --validate  # Validar YAML
pulse mock-api --spec pulse-mock.yaml             # Iniciar servidor
pulse --dry-run mock-api --spec pulse-mock.yaml   # Dry run
```

### Matching

- Exacto `/permisos`
- Parámetros `{id}` → regex `[^/]+`
- Regex manual: path iniciando con `^`

### Roadmap

| Feature                                     | Estado  |
| ------------------------------------------- | ------- |
| Hot reload                                  | Planned |
| Templates dinámicos (`{{now}}`, `{{uuid}}`) | Planned |
| Escenarios condicionales                    | Planned |
| Rate limiting / errores configurables       | Planned |
| Validación schemas                          | Planned |

Configura Cypress para apuntar a `http://localhost:7070/api` si quieres usarlo en tests.

## Configuration

### Execution Modes

#### Development Mode (default)

- Shows progress indicators
- Manages development server
- Provides user-friendly output
- Continues on test failures

#### CI Mode

- Optimized for continuous integration
- Minimal output
- Skips server management
- Fails fast on errors

#### Debug Mode

- Verbose logging
- Detailed execution information
- Progress indicators
- Debug-level metrics

### Server Configuration

```json
{
  "server": {
    "auto_start": true,
    "start_command": "npm run dev",
    "startup_timeout_ms": 30000,
    "health_check_retries": 5,
    "skip_health_check": false
  }
}
```

### Metrics & Reporting

#### Allure Reports

```json
{
  "metrics": {
    "allure": {
      "enabled": true,
      "report_dir": "cypress/reports/allure-results"
    }
  }
}
```

#### Sentry Integration

```json
{
  "metrics": {
    "sentry": {
      "enabled": true,
      "dsn": "https://your-sentry-dsn@sentry.io/123456",
      "environment": "test-automation"
    }
  }
}
```

#### Prometheus Metrics

```json
{
  "metrics": {
    "prometheus": {
      "enabled": true,
      "port": 9091
    }
  }
}
```

## Architecture

### Core Components

#### Adapters

- **CypressAdapter**: Manages Cypress test execution
- **GitAdapter**: Detects file changes using git
- **ServerManager**: Handles development server lifecycle
- **RouteIndexer**: Maps routes to test files
- **JUnitAdapter**: Processes and consolidates test reports

#### Metrics System

- **AllureAdapter**: Generates rich test reports
- **PrometheusAdapter**: Exposes performance metrics
- **SentryAdapter**: Tracks errors and performance issues

#### Utilities

- **FileWatcher**: Real-time file system monitoring
- **FileSystemUtils**: Robust file operations with validation
- **ProcessManager**: Cross-platform process management

### Execution Flow

1. **Configuration Loading**: Parse and validate pulse.json
2. **Change Detection**: Use git to identify modified files
3. **Impact Analysis**: Map changes to relevant test files using route index
4. **Server Management**: Start/check development server if needed
5. **Test Execution**: Run impacted tests with parallel workers
6. **Report Generation**: Consolidate results and generate reports
7. **Metrics Collection**: Send metrics to configured endpoints

## Advanced Usage

### Custom Test Patterns

```bash
# Override specs pattern
pulse --config pulse.json run
# With custom pattern in config:
# "specs_pattern": "**/*.{cy,spec}.{js,ts}"
```

### Environment-Specific Configurations

```bash
# Different configs for different environments
pulse --config pulse.ci.json --mode ci run
pulse --config pulse.dev.json --mode development watch
```

### Integration with CI/CD

```yaml
# GitHub Actions example
- name: Run Pulse Tests
  run: |
    pulse --mode ci --config pulse.ci.json run --workers 4
```

### Debugging

```bash
# Enable debug mode with verbose output
pulse --mode debug --verbose watch

# Dry run to see what would be executed
pulse --dry-run run
```

## Troubleshooting

### Common Issues

#### Server Not Starting

```bash
# Check server configuration
pulse --verbose run

# Test server command manually
npm run dev
```

#### No Tests Found

```bash
# Verify specs pattern
pulse --dry-run run

# Check file paths in config
ls -la app/routes/**/test/*.cy.ts
```

#### Git Integration Issues

```bash
# Ensure you're in a git repository
git status

# Check for uncommitted changes
git diff --name-only
```

#### Prometheus Port Conflicts

```bash
# If port 9091 is in use, change it in pulse.json
{
  "metrics": {
    "prometheus": {
      "enabled": true,
      "port": 9092  // Try alternative ports: 9092, 8080, 8090
    }
  }
}

# Check what's using the port
lsof -i :9091
```

### Performance Tuning

#### Optimize Worker Count

```bash
# Find optimal worker count (usually CPU cores - 1)
pulse run --workers $(nproc --ignore=1)
```

#### Adjust Debounce Time

```bash
# Reduce debounce for faster feedback
pulse watch --debounce-ms 500

# Increase for slower systems
pulse watch --debounce-ms 2000
```

## Development

### Building

```bash
cd utils/pulse
cargo build
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with coverage
cargo test --coverage
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### v0.1.0

- Initial release
- Smart test execution with impact analysis
- Parallel test running
- Server management
- Multiple reporting formats
- Watch mode with file monitoring
- NPM integration
- CI/CD support
