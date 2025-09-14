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
║              ⚡ EJECUTOR INTELIGENTE DE PRUEBAS IMPULSADO POR RUST ⚡               ║
║                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════╝
```

```
┌─ ESTADO DEL SISTEMA ─────────────────────────────────────────────────────────────────┐
│                                                                                      │
│  🎯 MISIÓN: Ejecución inteligente y de alto rendimiento para apps web modernas       │
│  🔧 MOTOR:  Rust + integración con Cypress                                           │
│  📊 MÉTRICAS: Monitorización en tiempo real (Prometheus, Sentry y Allure)            │
│  🚀 VELOCIDAD: Paralelismo + análisis de impacto                                      │
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

## 🚀 Qualitas Setup (host app)

- Directorio de trabajo: ejecuta los comandos desde `qualitas-cloud-2-frontend/`.
- Servicios mock: los YAML están en `mock_services/` (puertos 9011 y 9012).

Comandos útiles:

```bash
# 1) Validar YAMLs del simulador
npm run pulse:sim -- simulator validate --path mock_services --verbose

# 2) Arrancar simulador (Ctrl+C para parar)
npm run pulse:sim -- simulator start --services-dir mock_services

# 3) Ejecutar tests en vivo (watch)
npm run pulse -- watch

# 4) Modo avión (simulador + dev + watch + docs)
npm run start:airplane:watch

# 5) Integración real + contrato de login público + watch
npm run start:watch
```

### Grabar tráfico de API

```bash
# Proxy que captura peticiones y genera servicios YAML automáticamente
pulse simulator record --output services/ --url http://localhost:3000
```

### Importar/Exportar OpenAPI

```bash
# Generar un servicio YAML desde un spec OpenAPI
pulse simulator import --input openapi.yaml --output services/petstore.yaml

# Exportar un servicio mock existente a OpenAPI
pulse simulator export --input services/petstore.yaml --output openapi.yaml
```

### Exportar tipos TypeScript

```bash
# Generar interfaces TypeScript desde un servicio YAML
pulse simulator export-types --input services/petstore.yaml --output types.ts
```

Archivo generado (`types.ts`):

```ts
export interface paths {
  "/pets": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    /** List pets */
    get: {
      responses: {
        /** @description successful operation */
        200: {
          headers: { [name: string]: unknown };
          content?: never;
        };
      };
    };
  };
}
```

### Convertir desde Mockoon

```bash
# Generar un servicio YAML desde un archivo JSON de Mockoon
pulse simulator import-mockoon --input mockoon.json --output services/mockoon.yaml
```

Ejemplo de conversión:

```json
{
  "name": "Mockoon API",
  "port": 3000,
  "endpointPrefix": "/api",
  "routes": [
    {
      "method": "get",
      "endpoint": "/hello",
      "responses": [{ "statusCode": 200, "body": "{\"msg\":\"hola\"}" }]
    }
  ]
}
```

se convierte en:

```yaml
name: Mockoon API
server:
  port: 3000
  base_path: /api
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: |
          {"msg":"hola"}
```

### Convertir Postman/Insomnia

```bash
# Importar una colección de Postman
pulse simulator import-postman --input examples/postman-collection.json --output services/postman.yaml

# Importar una exportación de Insomnia
pulse simulator import-postman --input examples/insomnia-collection.json --output services/insomnia.yaml

# Exportar un servicio a colección Postman
pulse simulator export-postman --input services/postman.yaml --output postman-collection.json
```

Ejemplo (`examples/postman-collection.json`):

```json
{
  "info": { "name": "Sample Postman" },
  "item": [
    {
      "name": "Hello",
      "request": { "method": "GET", "url": { "raw": "http://localhost:3000/hello" } },
      "response": [{ "code": 200, "body": "{\"msg\":\"hi\"}" }]
    }
  ]
}
```

se convierte en:

```yaml
name: Sample Postman
server:
  base_path: /
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: |-
          {"msg":"hi"}
```

### Crear/Editar servicios del simulador

```bash
# Asistente interactivo para un nuevo servicio
pulse simulator new --output services

# Añadir un endpoint a un servicio existente
pulse simulator edit --input services/my-service.yaml
```

Ejemplo de YAML generado:

```yaml
name: my-service
server:
  port: 9000
  base_path: /api
endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        content_type: application/json
        body: |
          {"message":"hello"}
```

Endpoints de ejemplo (simulador):

- Login público: `http://localhost:9011/api/v1/public/login`
- Core público (logout): `http://localhost:9012/api/v1/logout`

Notas:
- `services_dir` en `pulse.json` del host debe ser `"mock_services"`.
- Los scripts `start:*` ya exportan `VITE_*` para apuntar a los mocks o al backend real según el caso.

### Data Bucket compartido

Define datos iniciales en memoria mediante `bucket` y accede a ellos con helpers:

```yaml
bucket:
  items: []
```

```handlebars
{{bucket.set "items" request.body}}
{{json (bucket.get "items")}}
```

### Mocking GraphQL

```yaml
name: graph-service
server:
  port: 9000
  base_path: /
graphql:
  schema_path: schemas/sample.graphql
  mocks:
    getUser: templates/get_user.json
```

Archivo de esquema (`schemas/sample.graphql`):

```graphql
type User { id: ID!, name: String! }
type Query {
  getUser(id: ID!): User
}
```

Cada operación se resuelve con la plantilla indicada y puede usar fixtures o variables de la petición (`request.body.variables`). Las peticiones POST a `/graphql` usan `operationName` para seleccionar el mock y una petición GET al mismo path devuelve el SDL del esquema.

## ⚡ Características Clave

```
╭─ MOTOR DE EJECUCIÓN ────────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🧠 ANÁLISIS DE IMPACTO  │ Cambios → tests relevantes automáticamente               │
│  ⚡ EJECUCIÓN EN PARALELO │ Grado de paralelismo configurable                        │
│  🔄 REINTENTOS           │ Manejo inteligente de tests inestables (flaky)           │
│  👁️  MODO WATCH           │ Observa cambios con debounce y ejecuta al vuelo          │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ GESTIÓN DE SERVIDOR ───────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🚀 AUTO-START          │ Arranque automático del servidor de desarrollo            │
│  💓 HEALTH CHECKS       │ Comprobaciones de salud configurables                     │
│  🔧 CONTROL DE PROCESOS │ Gestión limpia del ciclo de vida del servidor             │
│  ⏱️  TIMEOUTS            │ Esperas y reintentos configurables                       │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ REPORTING Y MÉTRICAS ──────────────────────────────────────────────────────────────╮
│                                                                                      │
│  📋 JUNIT REPORTS       │ Informes XML estándar con consolidación                   │
│  🎭 ALLURE              │ Reportes vistosos con capturas y logs                     │
│  📊 PROMETHEUS          │ Métricas de rendimiento y fiabilidad                      │
│  🔍 SENTRY              │ Trazado de errores y rendimiento                          │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ EXPERIENCIA DESARROLLADOR ─────────────────────────────────────────────────────────╮
│                                                                                      │
│  🎛️  MODOS DE EJECUCIÓN  │ CI, Development y Debug                                   │
│  🏃 DRY RUN             │ Simulación de ejecución sin correr pruebas                 │
│  📦 INTEGRACIÓN NPM     │ Configuración automática de scripts en package.json        │
│  🔍 LOGS DETALLADOS     │ Depuración y trazas detalladas                            │
│  🦐 SIMULADOR MOCK API  │ API local definida por YAML (endpoints, delays, escenarios)│
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

## 🛠️ Requisitos

- Rust 1.70+ (toolchain estable)
- Node.js 18+ (Cypress/TypeDoc)
- npm reciente
- Cypress 15+

## Configuración básica

Ejemplo de `pulse.json` mínimo:

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

### Scripts NPM

```bash
# Añade scripts Pulse al package.json
pulse setup-npm

# Solo mostrar instrucciones
pulse setup-npm --instructions-only
```

### Ejecutar pruebas

```bash
# Modo watch - ejecuta tests impactados al cambiar archivos
pulse watch

# Ejecutar toda la suite una vez
pulse run

# Ejecutar con configuración personalizada
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
- Headers requeridos usando `header_match`

Ejemplo de coincidencia por encabezados:

```yaml
endpoints:
  - method: GET
    path: /usuarios
    header_match:
      x-api-key: secret
    responses:
      200:
        content_type: application/json
        body: |
          {"status":"ok"}
```

La petición debe incluir `x-api-key: secret` para activar este endpoint.

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
pulse simulator validate --path mock_services --recursive --verbose

# Arrancar (usa pulse.json):
export PULSE_API_SIMULATOR=true   # también puedes habilitar en el JSON
pulse simulator start

# Estado / Parada
pulse simulator status
pulse simulator stop
```

3) Si una ruta no existe en tus mocks y quieres reenviarla a un backend real, añade `proxy_base_url` en el YAML del servicio:

```yaml
server:
  port: 8080
  base_path: /api/test
  proxy_base_url: https://api.example.com
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

#### Sistema de Métricas

- **AllureAdapter**: Genera reportes Allure
- **PrometheusAdapter**: Expone métricas de ejecución
- **SentryAdapter**: Reporta errores y rendimiento

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

### Depuración

```bash
# Activar modo debug con salida detallada
pulse --mode debug --verbose watch

# Simulación (dry-run) para ver qué se ejecutaría
pulse --dry-run run
```

## Resolución de Problemas

### Problemas Comunes

#### El servidor no arranca

```bash
# Verifica configuración del servidor
pulse --verbose run

# Prueba el comando manualmente
npm run dev
```

#### No se encuentran tests

```bash
# Verifica el patrón de especificaciones
pulse --dry-run run

# Comprueba rutas de archivos en la configuración
ls -la app/routes/**/test/*.cy.ts
```

#### Problemas con la integración Git

```bash
# Asegúrate de estar en un repo git
git status

# Cambios sin commitear
git diff --name-only
```

#### Conflictos de puerto Prometheus

```bash
# Si el puerto 9091 está en uso, cámbialo en pulse.json
{
  "metrics": {
    "prometheus": {
      "enabled": true,
  "port": 9092  // Alternativas: 9092, 8080, 8090
    }
  }
}

# Comprueba qué proceso usa el puerto
lsof -i :9091
```

### Rendimiento

#### Optimiza el número de workers

```bash
# Estima un valor razonable (p.ej. núm. de CPUs - 1)
pulse run --workers 4
```

#### Ajusta el tiempo de debounce

```bash
# Reduce para feedback más rápido
pulse watch --debounce-ms 500

# Aumenta en equipos más lentos
pulse watch --debounce-ms 2000
```

## GUI de Servicios Mock

La aplicación de escritorio construida con Tauri permite gestionar los servicios
mock definidos en YAML de forma visual. Para arrancarla ejecuta:

```bash
pulse gui
```

Desde la interfaz podrás iniciar y detener el simulador, editar archivos de
servicio y guardar los cambios directamente en YAML.

## Desarrollo

### Compilación

```bash
cargo build
```

### Pruebas

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Cobertura (si está configurada)
# cargo test --coverage
```

### Helpers de plantillas

Las plantillas Handlebars del simulador permiten generar datos de ejemplo y
consultar variables de entorno:

```hbs
{{faker "internet.email"}}   {{!-- correo electrónico realista --}}
{{env.MY_VARIABLE}}           {{!-- valor desde el entorno --}}
```

### Contribuir

1. Haz fork del repositorio
2. Crea una rama de feature
3. Añade tests para la nueva funcionalidad
4. Asegúrate de que todo pasa en CI
5. Abre un Pull Request

## Licencia

Proyecto con licencia MIT. Consulta el archivo LICENSE para más detalles.

## Cambios

### v0.1.0

- Versión inicial
- Ejecución inteligente con análisis de impacto
- Paralelización de pruebas
- Gestión de servidor de desarrollo
- Múltiples formatos de reporte
- Modo watch con monitorización de archivos
- Integración con NPM
- Soporte CI/CD
