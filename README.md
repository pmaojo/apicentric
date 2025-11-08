┌─ APICENTRIC ─────────────────────────────────────────────────────────────────────────┐
│                                                                                      │
│  🎯 MISIÓN: Simulador de APIs y testing de contratos desde línea de comandos        │
│  🔧 MOTOR:  Rust + configuración YAML para servicios mock                           │
│  � CLI:    Comandos para simulación, validación, generación y AI                   │
│  � P2P:    Colaboración distribuida y compartir servicios                          │
│                                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────## � Instalación

### Releases de GitHub

```bash
# Linux x64
curl -L https://github.com/pmaojo/apicentric/releases/latest/download/apicentric-linux-x64.tar.gz \
  | tar -xz && sudo mv apicentric /usr/local/bin

# macOS x64/ARM64
curl -L https://github.com/pmaojo/apicentric/releases/latest/download/apicentric-macos.tar.gz \
  | tar -xz && sudo mv apicentric /usr/local/bin

# Windows x64 (PowerShell)
Invoke-WebRequest -Uri "https://github.com/pmaojo/apicentric/releases/latest/download/apicentric-windows-x64.zip" -OutFile "apicentric.zip"
Expand-Archive -Path "apicentric.zip" -DestinationPath "."
```

### Gestores de paquetes

```bash
# Homebrew (macOS/Linux)
brew tap pmaojo/apicentric
brew install apicentric

# Winget (Windows)
winget install --id pmaojo.apicentric
```

### Compilación desde código fuente

```bash
git clone https://github.com/pmaojo/apicentric.git
cd apicentric
cargo build --release
sudo cp target/release/apicentric /usr/local/bin/
```

#### Opciones de Instalación con Cargo

Apicentric soporta diferentes configuraciones de características para optimizar el tamaño y tiempo de compilación:

```bash
# Instalación completa (recomendado)
cargo install apicentric

# CLI tools (simulador + testing de contratos + TUI)
cargo install apicentric --features cli-tools

# Solo simulador (más rápido)
cargo install apicentric --no-default-features --features simulator
```

**Características disponibles:**
- `simulator` - Motor de simulación de APIs (core)
- `tui` - Interfaz de usuario en terminal interactiva
- `contract-testing` - Testing de contratos con APIs reales
- `mock-data` - Generación de datos falsos con Faker
- `database` - Almacenamiento SQLite para estado persistente
- `file-watch` - Recarga automática al cambiar archivos

**Bundles de conveniencia:**
- `cli-tools` - Herramientas CLI esenciales (simulator + contract-testing + tui)
- `full` - Todas las características
- `minimal` - Solo el simulador (compilación más rápida)

Verifica la instalación:

```bash
apicentric --version
```

## 🚀 Comandos disponibles

Apicentric es una herramienta CLI para simulación de APIs, testing de contratos y generación de código:

```bash
# ============= SIMULADOR DE APIs =============
apicentric simulator start --services-dir mock_services --p2p   # Iniciar simulador con P2P
apicentric simulator validate --path mock_services --recursive  # Validar servicios YAML
apicentric simulator status --detailed                          # Estado de servicios activos
apicentric simulator logs my-service --limit 50                 # Ver logs de peticiones

# ============= IMPORTACIÓN/EXPORTACIÓN =============
apicentric simulator import-mockoon --input mockoon.json --output services/api.yaml
apicentric simulator import-postman --input collection.json --output services/api.yaml
apicentric simulator export-types --input services/api.yaml --output types.ts
apicentric simulator export-query --input services/api.yaml --output hooks.ts

# ============= GENERACIÓN ASISTIDA =============
apicentric ai generate "API de usuarios con GET /users y POST /users"

# ============= INTERFAZ TERMINAL =============
apicentric tui    # Dashboard interactivo en terminal
```

## 📋 Características principales

```
╭─ SIMULADOR DE APIS ─────────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🔧 SERVICIOS YAML      │ Definición declarativa de endpoints y respuestas         │
│  🚀 ARRANQUE RÁPIDO     │ Inicia múltiples servicios mock con un comando           │
│  📡 PROXY GRABACIÓN     │ Captura tráfico real y genera servicios automáticamente  │
│  🎛️  ESCENARIOS         │ Respuestas dinámicas según estado o condiciones          │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ GENERACIÓN DE CÓDIGO ──────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🎭 TIPOS TYPESCRIPT   │ Interfaces desde especificaciones YAML/OpenAPI            │
│  ⚛️  REACT QUERY       │ Hooks listos para usar con TanStack Query                 │
│  📮 POSTMAN/INSOMNIA   │ Collections para testing manual                           │
│  🔄 IMPORTACIÓN        │ Desde Mockoon, Postman, OpenAPI hacia YAML               │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ TESTING DE CONTRATOS ──────────────────────────────────────────────────────────────╮
│                                                                                      │
│  📝 REGISTRO           │ Gestiona contratos desde especificaciones de servicios    │
│  ✅ VALIDACIÓN         │ Compara mocks vs APIs reales para compatibilidad         │
│  📊 REPORTES           │ HTML con diferencias detectadas                           │
│  🎯 ESCENARIOS         │ Testing de casos específicos y edge cases                │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

╭─ COLABORACIÓN P2P ──────────────────────────────────────────────────────────────────╮
│                                                                                      │
│  🌐 LIBP2P             │ Compartir servicios entre equipos sin servidor central   │
│  🔗 CONEXIÓN REMOTA    │ Acceder a mocks de otros desarrolladores                 │
│  📡 EDICIÓN DISTRIBUTIVA│ Sincronización automática de cambios (CRDT)             │
│  🎨 TERMINAL DASHBOARD │ TUI para gestión visual desde consola                    │
│                                                                                      │
╰──────────────────────────────────────────────────────────────────────────────────────╯

## 🎯 Flujo de trabajo típico

### 1. Crear servicios mock

```bash
# Crear un nuevo servicio desde cero
apicentric simulator new --output services/

# O importar desde herramientas existentes
apicentric simulator import-mockoon --input mockoon.json --output services/api.yaml
apicentric simulator import-postman --input collection.json --output services/api.yaml
```

### 2. Arrancar el simulador

```bash
# Iniciar todos los servicios en el directorio
apicentric simulator start --services-dir services/

# Con colaboración P2P habilitada
apicentric simulator start --services-dir services/ --p2p

# Validar antes de arrancar
apicentric simulator validate --path services/ --recursive --verbose
```

### 3. Generar código para frontend

```bash
# Tipos TypeScript
apicentric simulator export-types --input services/api.yaml --output src/types.ts

# Hooks React Query
apicentric simulator export-query --input services/api.yaml --output src/api.ts

# Componente React de ejemplo
apicentric simulator export-view --input services/api.yaml --output src/ApiView.tsx
```

### 4. Testing de contratos

```bash
# Registrar contrato desde spec
apicentric contract register -n mi-api -s services/api.yaml

# Ejecutar validación completa
apicentric contract demo --contract-id <id> --with-simulator --html-report
```


## 🔧 Configuración inicial

### 1. Inicializar proyecto

```bash
# Crear estructura básica de directorios
mkdir -p services .apicentric/contracts

# Archivo de configuración mínimo (apicentric.json)
cat > apicentric.json << 'EOF'
{
  "services_dir": "services",
  "simulator": {
    "enabled": true,
    "port_range": { "start": 9000, "end": 9099 }
  },
  "ai": {
    "provider": "local"
  }
}
EOF
```

### 2. Verificar instalación

```bash
# Comprobar versión y comandos disponibles
apicentric --version
apicentric --help

# Validar configuración
apicentric --dry-run simulator validate --path services/ --verbose
```

### 3. Primer servicio mock

```bash
# Crear servicio básico interactivamente
apicentric simulator new --output services/

# O copiar ejemplo incluido
cp examples/user-api.yaml services/my-api.yaml

# Validar antes de usar
apicentric simulator validate --path services/ --recursive --verbose

# Iniciar simulador
apicentric simulator start --services-dir services/
```

## 🚀 Ejemplo de uso

- Directorio de trabajo: ejecuta los comandos desde `Ejemplo-cloud-2-frontend/`.
- Servicios mock: los YAML están en `mock_services/` (puertos 9011 y 9012).

Comandos útiles:

```bash
# 1) Validar YAMLs del simulador
npm run apicentric:sim -- simulator validate --path mock_services --verbose

# 2) Arrancar simulador (Ctrl+C para parar)
npm run apicentric:sim -- simulator start --services-dir mock_services

# 2b) Arrancar simulador con colaboración P2P
npm run apicentric:sim -- simulator start --services-dir mock_services --p2p

# 3) Convertir un proyecto Mockoon existente
npm run apicentric:sim -- simulator import-mockoon --input mockoon.json --output mock_services/mockoon.yaml

# 4) Grabar tráfico de una API en vivo
npm run apicentric:sim -- simulator record --output mock_services/ --url http://localhost:3000

# 5) Exportar tipos TypeScript
npm run apicentric:sim -- simulator export-types --input mock_services/petstore.yaml --output types.ts
```

### Grabar tráfico de API

```bash
# Proxy que captura peticiones y genera servicios YAML automáticamente
apicentric record --output services/ --url http://localhost:3000
```

### Filtrar y exportar logs del simulador

```bash
# Mostrar los últimos 50 logs GET con estado 200
apicentric logs my-service --limit 50 --method GET --status 200

# Filtrar por ruta y exportar a un archivo JSON
apicentric logs my-service --route /users --output logs.json
```

### Importar/Exportar OpenAPI

```bash
# Generar un servicio YAML desde un spec OpenAPI
apicentric import --input openapi.yaml --output services/petstore.yaml

# Exportar un servicio mock existente a OpenAPI
apicentric export --input services/petstore.yaml --output openapi.yaml
```

### Exportar tipos TypeScript

```bash
# Generar interfaces TypeScript desde un servicio YAML
apicentric export-types --input services/petstore.yaml --output types.ts
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
apicentric export-query --input services/petstore.yaml --output hooks.ts
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
apicentric import-mockoon --input mockoon.json --output services/mockoon.yaml
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
apicentric import-postman --input examples/postman-collection.json --output services/postman.yaml

# Importar una exportación de Insomnia
apicentric import-postman --input examples/insomnia-collection.json --output services/insomnia.yaml

# Exportar un servicio a colección Postman
apicentric export-postman --input services/postman.yaml --output postman-collection.json
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
apicentric new --output services

# Añadir un endpoint a un servicio existente
apicentric edit --input services/my-service.yaml
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
- `services_dir` en `apicentric.json` del host debe ser `"mock_services"`.
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

After installing `apicentric`, verify the CLI is available:

```bash
apicentric --help | head -n 5
```

You should see the `apicentric` banner with the version and a short usage summary, confirming the installation succeeded.

## Configuración básica

Ejemplo de `apicentric.json` mínimo:

```json
{
  "cypress_config_path": "cypress.config.ts",
  "base_url": "http://localhost:5173",
  "specs_pattern": "app/routes/**/test/*.cy.ts",
  "routes_dir": "app/routes",
  "specs_dir": "app/routes",
  "reports_dir": "cypress/reports",
  "index_cache_path": ".apicentric/route-index.json",
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
# Añade scripts apicentric al package.json
apicentric setup-npm

# Solo mostrar instrucciones
apicentric setup-npm --instructions-only
```

### Ejecutar simulador

```bash
# Iniciar el simulador con servicios YAML
apicentric start --services-dir services

# Validar servicios antes de iniciar
apicentric validate --path services

# Grabar tráfico de una API
apicentric record --output services/ --url http://localhost:3000
```

## Usage

### Command Line Interface

```bash
apicentric [OPTIONS] <COMMAND>

Commands:
  simulator   Manage API mocks (start, validate, record, import, export)
  setup-npm   Setup npm scripts for apicentric integration
  docs        Generate TypeScript documentation

Options:
  -c, --config <CONFIG>    Path to apicentric.json config file [default: apicentric.json]
      --dry-run           Enable dry-run mode (show what would be executed)
  -v, --verbose           Enable verbose output
  -h, --help              Print help

Simulador:
  apicentric start --services-dir services
  apicentric validate --path services
  apicentric record --output services/ --url http://localhost:3000
```

## 🦐 Mock API Simulator (Experimental)

YAML data-driven local API para simular servicios y trabajar offline.

### Ejemplo `apicentric-mock.yaml`

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
apicentric validate --path apicentric-mock.yaml    # Validar YAML
apicentric start --services apicentric-mock.yaml   # Iniciar servidor
apicentric --dry-run simulator start --services apicentric-mock.yaml   # Dry run
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
│  🛠️  Comando:      apicentric setup-npm                                                   │
│  🔎 Detección:      workspace, binarios locales, $HOME/.cargo, PATH                  │
│  🧩 Scripts:        "apicentric", "apicentric:sim"                                            │
│  🧪 Verificación:   --test para probar ejecución npm                                 │
│  📘 Ejemplos:       --examples muestra usos útiles                                   │
╰──────────────────────────────────────────────────────────────────────────────────────╯
```

### ¿Qué hace?

- Detecta si tienes `utils/apicentric` (workspace) o binarios compilados.
- Genera scripts npm recomendados sin pisar los existentes (a menos que uses `--force`).
- Imprime instrucciones cuando falta `package.json` o scripts.

### Detección inteligente del binario

El adaptador intenta, en orden:

1) `utils/apicentric/Cargo.toml` → usa `cargo run --manifest-path utils/apicentric/Cargo.toml --`
2) `./utils/apicentric/target/release|debug/apicentric` → usa el binario local
3) `which apicentric` en el `PATH`
4) `$HOME/.cargo/bin/apicentric`
5) Fallback: `cargo run --manifest-path utils/apicentric/Cargo.toml --`

Esto asegura que los scripts npm funcionen tanto en desarrollo como en CI sin fricción.

### Scripts que se añaden

```json
{
"scripts": {
    "apicentric": "<binario-detectado>",
    "apicentric:sim": "<binario-detectado> simulator"
  }
}
```

Puedes forzar la escritura con `--force`:

```bash
apicentric setup-npm --force
```

### Comprobación y ejemplos

```bash
# Verifica si los scripts están listos y ejecutables
apicentric setup-npm --test

# Muestra ejemplos de uso (útil para copy/paste)
apicentric setup-npm --examples
```

### Solución de problemas

- No hay `package.json`: crea uno (`npm init -y`) y vuelve a ejecutar `apicentric setup-npm`.
- El binario no aparece: compila apicentric (`cargo build`) o usa la ruta del workspace.
- NPM falla al ejecutar: verifica la salida de `--test` y revisa permisos de archivos.

## 🛰️ Ejemplo incluido: Mock Services (SWAPI)

Este repo trae ejemplos listos para usar de servicios mock basados en la API de Star Wars (SWAPI):

- `apicentric/examples/swapi-mock.yaml`: definición simple para el Mock Server directo (un archivo, rutas planas)
- `apicentric/examples/swapi-service.yaml`: definición de servicio para el API Simulator (plantillas, base_path, etc.)

### Opción A — Mock Server (archivo único)

Para un mock rápido sin directorios, puedes cargar un YAML y arrancar un servidor local con el módulo `mock` reexportado por apicentric:

```rust
// Cargo.toml: añade apicentric como dependencia si lo usas desde otro crate
// [dependencies]
// apicentric = { path = "utils/apicentric" }

use apicentric::mock::{load_spec, run_mock_server};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let spec = load_spec(Path::new("apicentric/examples/swapi-mock.yaml")).await?;
  run_mock_server(spec).await?; // Escucha en el puerto del YAML (p.ej. 8080)
  Ok(())
}
```

Rutas de ejemplo (si usas `swapi-mock.yaml`):

- GET http://127.0.0.1:8080/people/1/
- GET http://127.0.0.1:8080/people/
- GET http://127.0.0.1:8080/planets/1/

Esta vía usa el lector de YAML y el servidor mock definidos en:

- `apicentric/src/adapters/mock_server.rs:106` (lectura de YAML)
- `apicentric/src/adapters/mock_server.rs:120` (arranque del servidor)

### Opción B — API Simulator (directorio de servicios)

El API Simulator permite agrupar múltiples servicios YAML con base paths y plantillas. Para activarlo:

1) En `apicentric.json` habilita el simulador y apunta a tu carpeta de servicios. Importante: la ruta se resuelve respecto al directorio desde el que ejecutas apicentric (normalmente el root de tu app). En el caso de Ejemplo, apunta a `mock_services` del host app:

```json
{
  "simulator": {
    "enabled": true,
    "services_dir": "mock_services",
    "port_range": { "start": 9000, "end": 9099 }
  }
}
```

2) Arranca, para y consulta estado desde la CLI (ejecuta desde el root de `Ejemplo-cloud-2-frontend` para que `mock_services` se resuelva correctamente):

```bash
# Validar servicios (sin arrancar)
apicentric validate --path mock_services --recursive --verbose

# Arrancar (usa apicentric.json):
export PULSE_API_SIMULATOR=true   # también puedes habilitar en el JSON
apicentric start

# Estado / Parada
apicentric status
apicentric stop
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
  - `apicentric/src/simulator/manager.rs:43` (crea `ConfigLoader` con `services_dir`)
  - `apicentric/src/simulator/manager.rs:33` (constructor)
  - `apicentric/src/simulator/manager.rs:27` (start → `load_all_services()` y arranque)
- El `ConfigLoader` abre archivos YAML y valida cada servicio con `serde_yaml` + validaciones:
  - `apicentric/src/simulator/config.rs:578` (lee YAML del disco)
  - `apicentric/src/simulator/config.rs:586` (parsea con `serde_yaml`)
  - `apicentric/src/simulator/config.rs:598` (valida estructura)
  - `apicentric/src/simulator/config.rs:615` y `660` (escaneo recursivo con estadísticas)

Además, puedes ejecutar una validación aislada contra un directorio sin necesidad de configurar `apicentric.json` (útil para CI):

```bash
apicentric validate --path apicentric/examples --recursive --verbose
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

> Resultado: apicentric está listo para integrarse en monorepos JavaScript/TypeScript con una DX de primera, tiempos de ejecución bajos y visibilidad operativa de nivel producción.

#### Utilities

- **FileWatcher**: Real-time file system monitoring
- **FileSystemUtils**: Robust file operations with validation
- **ProcessManager**: Cross-platform process management

### Execution Flow

1. **Configuration Loading**: Parse and validate apicentric.json
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
apicentric --config apicentric.ci.json simulator start --services-dir services
apicentric --config apicentric.dev.json simulator start --services-dir services
```

### Integración con CI/CD

```yaml
# GitHub Actions example
- name: Start apicentric
  run: |
    apicentric --config apicentric.ci.json simulator start --services-dir services
```

### Depuración

```bash
# Activar modo debug con salida detallada
apicentric --dry-run simulator start --services-dir services
```

## Resolución de Problemas

### Problemas Comunes

#### El servidor no arranca

```bash
# Verifica configuración del servidor
apicentric --verbose run

# Prueba el comando manualmente
npm run dev
```

#### No se encuentran tests

```bash
# Verifica el patrón de especificaciones
apicentric --dry-run run

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
# Si el puerto 9091 está en uso, cámbialo en apicentric.json
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
apicentric gui
```

Desde la interfaz podrás iniciar y detener el simulador, editar archivos de
servicio y guardar los cambios directamente en YAML.

## ✅ Testing y calidad

### Suite de tests integrada

Apicentric incluye tests comprehensivos para garantizar la estabilidad:

```bash
# Tests del simulador de APIs
cargo test --test simulator_integration

# Tests del sistema de plugins
cargo test --test plugin_system  

# Tests de carga de especificaciones YAML
cargo test --test service_spec_loader

# Tests de comandos CLI y contexto
cargo test --test cli_commands

# Ejecutar todos los tests
cargo test --all
```

### Validación de servicios

```bash
# Validar servicio específico con detalles
apicentric simulator validate --path services/api.yaml --verbose

# Validar directorio completo recursivamente  
apicentric simulator validate --path services/ --recursive --verbose

# Modo dry-run para ver qué se ejecutaría
apicentric --dry-run simulator validate --path services/
```

### Performance de compilación

```bash
# Build rápido para desarrollo
cargo build

# Build optimizado para producción
cargo build --release

# Verificación sin compilar (muy rápido)
cargo check
```

### Ejemplos funcionales

Encuentra ejemplos completos en `examples/`:
- `user-api.yaml` - API básica de usuarios con CRUD
- `ecommerce-api.yaml` - API de e-commerce con productos y órdenes  
- `quickstart/` - Tutorial completo paso a paso

```bash
# Probar ejemplo de usuario
cp examples/user-api.yaml services/
apicentric simulator start --services-dir services/

# En otra terminal
curl http://localhost:9001/api/v1/users
```

## 🧭 Desarrollo

### Contribuir

1. Fork del repositorio
2. Crear rama de feature: `git checkout -b feature/nueva-funcionalidad`
3. Commit cambios: `git commit -am 'Añadir nueva funcionalidad'`
4. Push a la rama: `git push origin feature/nueva-funcionalidad`  
5. Crear Pull Request

### Compilación desde fuente

```bash
# Clonar repositorio
git clone https://github.com/pmaojo/apicentric.git
cd apicentric

# Instalar dependencias y compilar
cargo build --release

# Ejecutar tests
cargo test

# Instalar binario
cargo install --path .
```

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
