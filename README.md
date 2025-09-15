┌─ ESTADO DEL SISTEMA ─────────────────────────────────────────────────────────────────┐
│                                                                                      │
│  🎯 MISIÓN: Simulación y mock de APIs para desarrollo rápido                         │
│  🔧 MOTOR:  Rust + servicios YAML                                                    │
│  📊 MÉTRICAS: Monitorización en tiempo real (Prometheus, Sentry y Allure)            │
│  🚀 VELOCIDAD: Conversión y grabación automáticas                                    │
│                                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────
## 🛠️ Instalación

Para instalar la versión más reciente de `mockforge`, ejecuta uno de los scripts de instalación incluidos:

- **Linux/macOS**: `./scripts/install.sh`
- **Windows (PowerShell)**: `./scripts/install.ps1`

Cada script detecta automáticamente tu sistema operativo y arquitectura, descarga el binario adecuado desde las últimas releases y lo coloca en una ubicación habitual (`/usr/local/bin` o `%UserProfile%\bin`).

## 📦 Instalación

### Homebrew (macOS)

```bash
brew tap your-org/pulse
brew install mockforge
```

### Windows (winget)

```powershell
winget install --id your-org.mockforge
```

## Installation

- **Linux**

  ```bash
  curl -L <release_url>/mockforge-linux-x64.tar.gz | tar -xz && sudo mv mockforge /usr/local/bin
  ```

- **macOS**

  ```bash
  brew install <tap>/mockforge
  ```

- **Windows**
  1. Download `mockforge-windows-x64.zip` from `<release_url>`.
  2. Extract `mockforge.exe` and add its folder to your `PATH`.

```bash
mockforge --version
```

## ✨ Guía Rápida

```
╭──────────────────────────────────────────────────────────────────────────────╮
│ 1) Configura mockforge.json                                                       │
│ 2) Integra scripts npm (mockforge setup-npm)                                      │
│ 3) Arranca el simulador y gestiona servicios mock                             │
╰──────────────────────────────────────────────────────────────────────────────╯
```

```bash
# Inicializa config por defecto (si aún no tienes mockforge.json)
mockforge init

# Ajusta rutas y directorios de servicios en mockforge.json

# Añade scripts npm automáticamente
mockforge setup-npm

# Inicia el simulador con tus servicios mock
npm run mockforge:sim -- simulator start --services-dir mock_services

# Especifica una ruta de base de datos SQLite para almacenar servicios y logs
npm run mockforge:sim -- simulator start --services-dir mock_services --db-path pulse.db

# Habilita la edición colaborativa distribuida entre pares
npm run mockforge:sim -- simulator start --services-dir mock_services --p2p

# Convierte un archivo Mockoon a YAML
mockforge import-mockoon --input mockoon.json --output services/mockoon.yaml

# Graba tráfico de una API real
mockforge record --output services/ --url http://localhost:3000

# Exporta interfaces TypeScript
mockforge export-types --input services/petstore.yaml --output types.ts
```

## Instalación y uso en Node.js

```bash
npm install mockforge
```

```javascript
const { greet } = require('mockforge');
console.log(greet('World'));
```

### Edición colaborativa P2P

Arranca el simulador con la bandera `--p2p` para descubrir automáticamente a otros
peers en la red local (mDNS) y compartir cambios de los servicios mediante CRDTs.
Las modificaciones en los archivos YAML se fusionan y propagan sin necesidad de
un servidor central.

### Compartir servicios via libp2p

Pulsa permite exponer un servicio en ejecución para que otros peers consuman el
mock de forma remota.

```bash
# En el host que tiene el simulador corriendo
mockforge simulator share my-service
# Muestra Peer ID y token de acceso

# En otro equipo
mockforge simulator connect <peer-id> --service my-service --port 8080 --token <token>
# Abre un proxy local en http://localhost:8080
```

> **Seguridad:** El token se debe compartir solo con colaboradores de
confianza. Actualmente cualquier peer con el token puede conectarse; para un
control más fino pueden implementarse listas de peers permitidos.


### Generación asistida por IA

Configura un proveedor en `mockforge.json` para generar servicios desde prompts en
lenguaje natural. Ejemplo con un modelo local:

```json
{
  "ai": { "provider": "local", "model_path": "models/llama.bin" }
}
```

O utilizando la API de OpenAI:

```json
{
  "ai": { "provider": "openai", "api_key": "sk-…", "model": "gpt-3.5-turbo" }
}
```

Para generar un servicio YAML y aplicarlo al proyecto activo:

```bash
mockforge ai generate "Servicio de usuarios con GET /users"
```

Usar el proveedor local mantiene todos los datos en tu máquina. Con OpenAI, el
prompt y el resultado se envían al servicio externo.

## 🚀 Qualitas Setup (host app)

- Directorio de trabajo: ejecuta los comandos desde `qualitas-cloud-2-frontend/`.
- Servicios mock: los YAML están en `mock_services/` (puertos 9011 y 9012).

Comandos útiles:

```bash
# 1) Validar YAMLs del simulador
npm run mockforge:sim -- simulator validate --path mock_services --verbose

# 2) Arrancar simulador (Ctrl+C para parar)
npm run mockforge:sim -- simulator start --services-dir mock_services

# 2b) Arrancar simulador con colaboración P2P
npm run mockforge:sim -- simulator start --services-dir mock_services --p2p

# 3) Convertir un proyecto Mockoon existente
npm run mockforge:sim -- simulator import-mockoon --input mockoon.json --output mock_services/mockoon.yaml

# 4) Grabar tráfico de una API en vivo
npm run mockforge:sim -- simulator record --output mock_services/ --url http://localhost:3000

# 5) Exportar tipos TypeScript
npm run mockforge:sim -- simulator export-types --input mock_services/petstore.yaml --output types.ts
```

### Grabar tráfico de API

```bash
# Proxy que captura peticiones y genera servicios YAML automáticamente
mockforge record --output services/ --url http://localhost:3000
```

### Filtrar y exportar logs del simulador

```bash
# Mostrar los últimos 50 logs GET con estado 200
mockforge logs my-service --limit 50 --method GET --status 200

# Filtrar por ruta y exportar a un archivo JSON
mockforge logs my-service --route /users --output logs.json
```

### Importar/Exportar OpenAPI

```bash
# Generar un servicio YAML desde un spec OpenAPI
mockforge import --input openapi.yaml --output services/petstore.yaml

# Exportar un servicio mock existente a OpenAPI
mockforge export --input services/petstore.yaml --output openapi.yaml
```

### Exportar tipos TypeScript

```bash
# Generar interfaces TypeScript desde un servicio YAML
mockforge export-types --input services/petstore.yaml --output types.ts
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

### Exportar hooks de TanStack Query

```bash
# Generar hooks React Query desde un servicio YAML
mockforge export-query --input services/petstore.yaml --output hooks.ts
```

Archivo generado (`hooks.ts`):

```ts
import { useQuery, useMutation } from '@tanstack/react-query';

export function usePetsQuery(baseUrl: string) {
  return useQuery(['GET','/pets'], () => fetch(`${baseUrl}/api/pets`).then(res => res.json()));
}

export function usePostPetsMutation(baseUrl: string) {
  return useMutation((body: any) =>
    fetch(`${baseUrl}/api/pets`, { method: 'POST', body: JSON.stringify(body) }).then(res => res.json())
  );
}
```

En una aplicación React:

```tsx
import { usePetsQuery, usePostPetsMutation } from './hooks';

function Pets() {
  const pets = usePetsQuery('/api');
  const addPet = usePostPetsMutation('/api');
  // ...
}
```

### Convertir desde Mockoon

```bash
# Generar un servicio YAML desde un archivo JSON de Mockoon
mockforge import-mockoon --input mockoon.json --output services/mockoon.yaml
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
mockforge import-postman --input examples/postman-collection.json --output services/postman.yaml

# Importar una exportación de Insomnia
mockforge import-postman --input examples/insomnia-collection.json --output services/insomnia.yaml

# Exportar un servicio a colección Postman
mockforge export-postman --input services/postman.yaml --output postman-collection.json
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
mockforge new --output services

# Añadir un endpoint a un servicio existente
mockforge edit --input services/my-service.yaml
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
- `services_dir` en `mockforge.json` del host debe ser `"mock_services"`.
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

### Scripts JS/WASM personalizados

Ejecuta lógica dinámica antes de renderizar la respuesta. El script recibe el contexto
de la petición y puede devolver un objeto con datos que se guardan en `runtime`.

```yaml
responses:
  200:
    content_type: application/json
    body: '{"id": "{{ runtime.userId }}"}'
    script: scripts/extract_id.js
```

Archivo `scripts/extract_id.js`:

```javascript
// ctx => { request, params, fixtures, runtime }
({ userId: ctx.request.body.id })
```

Los scripts se ejecutan en un entorno aislado sin acceso a red ni sistema de archivos,
proporcionando un sandbox seguro. También se pueden cargar módulos WASM desde el
script utilizando la API estándar de WebAssembly.

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

## 📦 Installation

After installing `mockforge`, verify the CLI is available:

```bash
mockforge --help | head -n 5
```

You should see the `mockforge` banner with the version and a short usage summary, confirming the installation succeeded.

## Configuración básica

Ejemplo de `mockforge.json` mínimo:

```json
{
  "cypress_config_path": "cypress.config.ts",
  "base_url": "http://localhost:5173",
  "specs_pattern": "app/routes/**/test/*.cy.ts",
  "routes_dir": "app/routes",
  "specs_dir": "app/routes",
  "reports_dir": "cypress/reports",
  "index_cache_path": ".mockforge/route-index.json",
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
# Añade scripts MockForge al package.json
mockforge setup-npm

# Solo mostrar instrucciones
mockforge setup-npm --instructions-only
```

### Ejecutar simulador

```bash
# Iniciar el simulador con servicios YAML
mockforge start --services-dir services

# Validar servicios antes de iniciar
mockforge validate --path services

# Grabar tráfico de una API
mockforge record --output services/ --url http://localhost:3000
```

## Usage

### Command Line Interface

```bash
mockforge [OPTIONS] <COMMAND>

Commands:
  simulator   Manage API mocks (start, validate, record, import, export)
  setup-npm   Setup npm scripts for mockforge integration
  docs        Generate TypeScript documentation

Options:
  -c, --config <CONFIG>    Path to mockforge.json config file [default: mockforge.json]
      --dry-run           Enable dry-run mode (show what would be executed)
  -v, --verbose           Enable verbose output
  -h, --help              Print help

Simulador:
  mockforge start --services-dir services
  mockforge validate --path services
  mockforge record --output services/ --url http://localhost:3000
```

## 🦐 Mock API Simulator (Experimental)

YAML data-driven local API para simular servicios y trabajar offline.

### Ejemplo `mockforge-mock.yaml`

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
mockforge validate --path mockforge-mock.yaml    # Validar YAML
mockforge start --services mockforge-mock.yaml   # Iniciar servidor
mockforge --dry-run simulator start --services mockforge-mock.yaml   # Dry run
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


## 🔗 Integración con NPM (setup-npm)

```
╭─ AUTOMATIZA TU FLUJO ───────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🛠️  Comando:      mockforge setup-npm                                                   │
│  🔎 Detección:      workspace, binarios locales, $HOME/.cargo, PATH                  │
│  🧩 Scripts:        "mockforge", "mockforge:sim"                                            │
│  🧪 Verificación:   --test para probar ejecución npm                                 │
│  📘 Ejemplos:       --examples muestra usos útiles                                   │
╰──────────────────────────────────────────────────────────────────────────────────────╯
```

### ¿Qué hace?

- Detecta si tienes `utils/mockforge` (workspace) o binarios compilados.
- Genera scripts npm recomendados sin pisar los existentes (a menos que uses `--force`).
- Imprime instrucciones cuando falta `package.json` o scripts.

### Detección inteligente del binario

El adaptador intenta, en orden:

1) `utils/mockforge/Cargo.toml` → usa `cargo run --manifest-path utils/mockforge/Cargo.toml --`
2) `./utils/mockforge/target/release|debug/mockforge` → usa el binario local
3) `which mockforge` en el `PATH`
4) `$HOME/.cargo/bin/mockforge`
5) Fallback: `cargo run --manifest-path utils/mockforge/Cargo.toml --`

Esto asegura que los scripts npm funcionen tanto en desarrollo como en CI sin fricción.

### Scripts que se añaden

```json
{
"scripts": {
    "mockforge": "<binario-detectado>",
    "mockforge:sim": "<binario-detectado> simulator"
  }
}
```

Puedes forzar la escritura con `--force`:

```bash
mockforge setup-npm --force
```

### Comprobación y ejemplos

```bash
# Verifica si los scripts están listos y ejecutables
mockforge setup-npm --test

# Muestra ejemplos de uso (útil para copy/paste)
mockforge setup-npm --examples
```

### Solución de problemas

- No hay `package.json`: crea uno (`npm init -y`) y vuelve a ejecutar `mockforge setup-npm`.
- El binario no aparece: compila MockForge (`cargo build`) o usa la ruta del workspace.
- NPM falla al ejecutar: verifica la salida de `--test` y revisa permisos de archivos.

## 🛰️ Ejemplo incluido: Mock Services (SWAPI)

Este repo trae ejemplos listos para usar de servicios mock basados en la API de Star Wars (SWAPI):

- `mockforge/examples/swapi-mock.yaml`: definición simple para el Mock Server directo (un archivo, rutas planas)
- `mockforge/examples/swapi-service.yaml`: definición de servicio para el API Simulator (plantillas, base_path, etc.)

### Opción A — Mock Server (archivo único)

Para un mock rápido sin directorios, puedes cargar un YAML y arrancar un servidor local con el módulo `mock` reexportado por MockForge:

```rust
// Cargo.toml: añade mockforge como dependencia si lo usas desde otro crate
// [dependencies]
// mockforge = { path = "utils/mockforge" }

use mockforge::mock::{load_spec, run_mock_server};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let spec = load_spec(Path::new("mockforge/examples/swapi-mock.yaml")).await?;
  run_mock_server(spec).await?; // Escucha en el puerto del YAML (p.ej. 8080)
  Ok(())
}
```

Rutas de ejemplo (si usas `swapi-mock.yaml`):

- GET http://127.0.0.1:8080/people/1/
- GET http://127.0.0.1:8080/people/
- GET http://127.0.0.1:8080/planets/1/

Esta vía usa el lector de YAML y el servidor mock definidos en:

- `mockforge/src/adapters/mock_server.rs:106` (lectura de YAML)
- `mockforge/src/adapters/mock_server.rs:120` (arranque del servidor)

### Opción B — API Simulator (directorio de servicios)

El API Simulator permite agrupar múltiples servicios YAML con base paths y plantillas. Para activarlo:

1) En `mockforge.json` habilita el simulador y apunta a tu carpeta de servicios. Importante: la ruta se resuelve respecto al directorio desde el que ejecutas MockForge (normalmente el root de tu app). En el caso de Qualitas, apunta a `mock_services` del host app:

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
mockforge validate --path mock_services --recursive --verbose

# Arrancar (usa mockforge.json):
export PULSE_API_SIMULATOR=true   # también puedes habilitar en el JSON
mockforge start

# Estado / Parada
mockforge status
mockforge stop
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
  - `mockforge/src/simulator/manager.rs:43` (crea `ConfigLoader` con `services_dir`)
  - `mockforge/src/simulator/manager.rs:33` (constructor)
  - `mockforge/src/simulator/manager.rs:27` (start → `load_all_services()` y arranque)
- El `ConfigLoader` abre archivos YAML y valida cada servicio con `serde_yaml` + validaciones:
  - `mockforge/src/simulator/config.rs:578` (lee YAML del disco)
  - `mockforge/src/simulator/config.rs:586` (parsea con `serde_yaml`)
  - `mockforge/src/simulator/config.rs:598` (valida estructura)
  - `mockforge/src/simulator/config.rs:615` y `660` (escaneo recursivo con estadísticas)

Además, puedes ejecutar una validación aislada contra un directorio sin necesidad de configurar `mockforge.json` (útil para CI):

```bash
mockforge validate --path mockforge/examples --recursive --verbose
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

> Resultado: MockForge está listo para integrarse en monorepos JavaScript/TypeScript con una DX de primera, tiempos de ejecución bajos y visibilidad operativa de nivel producción.

#### Utilities

- **FileWatcher**: Real-time file system monitoring
- **FileSystemUtils**: Robust file operations with validation
- **ProcessManager**: Cross-platform process management

### Execution Flow

1. **Configuration Loading**: Parse and validate mockforge.json
2. **Change Detection**: Use git to identify modified files
3. **Impact Analysis**: Map changes to relevant test files using route index
4. **Server Management**: Start/check development server if needed
5. **Test Execution**: Run impacted tests with parallel workers
6. **Report Generation**: Consolidate results and generate reports
7. **Metrics Collection**: Send metrics to configured endpoints

## Advanced Usage

### Custom Test Patterns

### Configuraciones por entorno

```bash
# Diferentes configs para distintos entornos
mockforge --config mockforge.ci.json simulator start --services-dir services
mockforge --config mockforge.dev.json simulator start --services-dir services
```

### Integración con CI/CD

```yaml
# GitHub Actions example
- name: Start MockForge
  run: |
    mockforge --config mockforge.ci.json simulator start --services-dir services
```

### Depuración

```bash
# Activar modo debug con salida detallada
mockforge --dry-run simulator start --services-dir services
```

## Resolución de Problemas

### Problemas Comunes

#### El servidor no arranca

```bash
# Verifica configuración del servidor
mockforge --verbose run

# Prueba el comando manualmente
npm run dev
```

#### No se encuentran tests

```bash
# Verifica el patrón de especificaciones
mockforge --dry-run run

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
# Si el puerto 9091 está en uso, cámbialo en mockforge.json
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

## GUI de Servicios Mock

La aplicación de escritorio construida con Tauri permite gestionar los servicios
mock definidos en YAML de forma visual. Para arrancarla ejecuta:

```bash
mockforge gui
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

### Escenarios y rotación de respuestas

Cada endpoint puede definir múltiples escenarios dentro de la clave `scenarios`.
Cuando un escenario no posee `name` ni `conditions`, puede emplearse para
rotar respuestas automáticamente.

```yaml
scenarios:
  - strategy: sequential # también "random"
    response:
      status: 200
      body: "ok"
  - response:
      status: 500
      body: "error"
```

Con la estrategia `sequential` las respuestas se devuelven en orden y vuelven al
inicio al llegar al final. Con `random` se elige una respuesta aleatoriamente en
cada petición.

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
- Simulador de APIs definido en YAML
- Grabación de tráfico y generación automática de servicios
- Conversión desde Mockoon y Postman/Insomnia
- Exportación de especificaciones OpenAPI y tipos TypeScript
- GUI para gestionar servicios mock
