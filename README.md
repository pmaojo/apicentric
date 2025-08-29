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

## ✨ Guía Rápida

```
╭──────────────────────────────────────────────────────────────────────────────╮
│ 1) Configura pulse.json                                                       │
│ 2) Integra scripts npm (pulse setup-npm)                                      │
│ 3) Ejecuta: npm run pulse -- run | watch                                      │
╰──────────────────────────────────────────────────────────────────────────────╯
```

```bash
# Inicializa config por defecto (si aún no tienes pulse.json)
pulse init

# Revisa y ajusta rutas/patrones/baseUrl en pulse.json

# Añade scripts npm automáticamente
pulse setup-npm

# Ejecuta todo
npm run pulse -- run

# Observa cambios y ejecuta impactados
npm run pulse:watch
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

## 🔗 Integración con NPM (setup-npm)

```
╭─ AUTOMATIZA TU FLUJO ───────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🛠️  Comando:      pulse setup-npm                                                   │
│  🔎 Detección:      workspace, binarios locales, $HOME/.cargo, PATH                  │
│  🧩 Scripts:        "pulse", "pulse:watch"                                          │
│  🧪 Verificación:   --test para probar ejecución npm                                 │
│  📘 Ejemplos:       --examples muestra usos útiles                                   │
╰──────────────────────────────────────────────────────────────────────────────────────╯
```

### ¿Qué hace?

- Detecta si tienes `utils/pulse` (workspace) o binarios compilados.
- Genera scripts npm recomendados sin pisar los existentes (a menos que uses `--force`).
- Imprime instrucciones cuando falta `package.json` o scripts.

### Detección inteligente del binario

El adaptador intenta, en orden:

1) `utils/pulse/Cargo.toml` → usa `cargo run --manifest-path utils/pulse/Cargo.toml --`
2) `./utils/pulse/target/release|debug/pulse` → usa el binario local
3) `which pulse` en el `PATH`
4) `$HOME/.cargo/bin/pulse`
5) Fallback: `cargo run --manifest-path utils/pulse/Cargo.toml --`

Esto asegura que los scripts npm funcionen tanto en desarrollo como en CI sin fricción.

### Scripts que se añaden

```json
{
  "scripts": {
    "pulse": "<binario-detectado>",
    "pulse:watch": "<binario-detectado> watch"
  }
}
```

Puedes forzar la escritura con `--force`:

```bash
pulse setup-npm --force
```

### Comprobación y ejemplos

```bash
# Verifica si los scripts están listos y ejecutables
pulse setup-npm --test

# Muestra ejemplos de uso (útil para copy/paste)
pulse setup-npm --examples
```

### Solución de problemas

- No hay `package.json`: crea uno (`npm init -y`) y vuelve a ejecutar `pulse setup-npm`.
- El binario no aparece: compila Pulse (`cargo build`) o usa la ruta del workspace.
- NPM falla al ejecutar: verifica la salida de `--test` y revisa permisos de archivos.

## 🛰️ Ejemplo incluido: Mock Services (SWAPI)

Este repo trae ejemplos listos para usar de servicios mock basados en la API de Star Wars (SWAPI):

- `pulse/examples/swapi-mock.yaml`: definición simple para el Mock Server directo (un archivo, rutas planas)
- `pulse/examples/swapi-service.yaml`: definición de servicio para el API Simulator (plantillas, base_path, etc.)

### Opción A — Mock Server (archivo único)

Para un mock rápido sin directorios, puedes cargar un YAML y arrancar un servidor local con el módulo `mock` reexportado por Pulse:

```rust
// Cargo.toml: añade pulse como dependencia si lo usas desde otro crate
// [dependencies]
// pulse = { path = "utils/pulse" }

use pulse::mock::{load_spec, run_mock_server};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let spec = load_spec(Path::new("pulse/examples/swapi-mock.yaml")).await?;
  run_mock_server(spec).await?; // Escucha en el puerto del YAML (p.ej. 8080)
  Ok(())
}
```

Rutas de ejemplo (si usas `swapi-mock.yaml`):

- GET http://127.0.0.1:8080/people/1/
- GET http://127.0.0.1:8080/people/
- GET http://127.0.0.1:8080/planets/1/

Esta vía usa el lector de YAML y el servidor mock definidos en:

- `pulse/src/adapters/mock_server.rs:106` (lectura de YAML)
- `pulse/src/adapters/mock_server.rs:120` (arranque del servidor)

### Opción B — API Simulator (directorio de servicios)

El API Simulator permite agrupar múltiples servicios YAML con base paths y plantillas. Para activarlo:

1) En `pulse.json` habilita el simulador y apunta a tu carpeta de servicios. Importante: la ruta se resuelve respecto al directorio desde el que ejecutas Pulse (normalmente el root de tu app). En el caso de Qualitas, apunta a `mock_services` del host app:

```json
{
  "simulator": {
    "enabled": true,
    "services_dir": "mock_services",
    "port_range": { "start": 9000, "end": 9099 }
  }
}
```

2) Arranca, para y consulta estado desde la CLI (ejecuta desde el root de `qualitas-cloud-2-frontend` para que `mock_services` se resuelva correctamente):

```bash
# Validar servicios (sin arrancar)
pulse simulator validate --path pulse/examples

# Arrancar (usa pulse.json):
export PULSE_API_SIMULATOR=true   # también puedes habilitar en el JSON
pulse simulator start

# Estado / Parada
pulse simulator status
pulse simulator stop
```

Rutas de ejemplo (según `swapi-service.yaml` y el base_path configurado):

- GET http://127.0.0.1:9011/api/v1/public/people/1/
- GET http://127.0.0.1:9011/api/v1/public/people/

Cómo sabemos que “sí los está leyendo” tras el refactor:

- El manager carga todos los servicios desde `services_dir` a través del `ConfigLoader` y los registra:
  - `pulse/src/simulator/manager.rs:43` (crea `ConfigLoader` con `services_dir`)
  - `pulse/src/simulator/manager.rs:33` (constructor)
  - `pulse/src/simulator/manager.rs:27` (start → `load_all_services()` y arranque)
- El `ConfigLoader` abre archivos YAML y valida cada servicio con `serde_yaml` + validaciones:
  - `pulse/src/simulator/config.rs:578` (lee YAML del disco)
  - `pulse/src/simulator/config.rs:586` (parsea con `serde_yaml`)
  - `pulse/src/simulator/config.rs:598` (valida estructura)
  - `pulse/src/simulator/config.rs:615` y `660` (escaneo recursivo con estadísticas)

Además, puedes ejecutar una validación aislada contra un directorio sin necesidad de configurar `pulse.json` (útil para CI):

```bash
pulse simulator validate --path pulse/examples --recursive --verbose
```

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

## 🔒 Seguridad y Buenas Prácticas

- Evita credenciales hardcodeadas en comandos o ejemplos. Usa variables de entorno y léelas en tiempo de ejecución.
- Mantén fuera del control de versiones artefactos de build y archivos temporales (`target/`, `.DS_Store`, `*.zip`).
- Activa calidad en CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.

## 🧭 Resumen de Auditoría (alto nivel)

- Arquitectura limpia por capas (dominio/puertos/adaptadores) y builder de contexto sólido.
- Validación de configuración y mensajería de errores clara con sugerencias.
- Integración npm robusta: detección multinivel del binario, escritura segura de scripts y verificación (`--test`).
- Recomendaciones: unificar tipos de error, homogeneizar logs con `tracing`, reemplazar dependencias externas frágiles (p.ej. `curl`) por clientes HTTP embebidos.

> Resultado: Pulse está listo para integrarse en monorepos JavaScript/TypeScript con una DX de primera, tiempos de ejecución bajos y visibilidad operativa de nivel producción.
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
