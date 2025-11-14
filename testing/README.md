# 🧪 Apicentric E2E Testing Suite

Sistema completo de testing end-to-end para Apicentric usando Playwright. Este sistema permite probar tanto el frontend (webui) como el backend de forma integrada.

## 📋 Tabla de Contenidos

- [Características](#-características)
- [Estructura del Proyecto](#-estructura-del-proyecto)
- [Instalación y Configuración](#-instalación-y-configuración)
- [Ejecución de Tests](#-ejecución-de-tests)
- [Tipos de Tests](#-tipos-de-tests)
- [Configuración](#-configuración)
- [Desarrollo de Tests](#-desarrollo-de-tests)
- [CI/CD Integration](#-cicd-integration)
- [Troubleshooting](#-troubleshooting)

## 🚀 Características

- **Testing Completo**: Tests de frontend, backend y integración
- **Multi-browser**: Chromium, Firefox, Safari (Webkit)
- **Headless/Headed**: Ejecución con o sin interfaz gráfica
- **Paralelo**: Ejecución paralela de tests para mayor velocidad
- **Auto-setup**: Inicia backend y frontend automáticamente
- **Cleanup**: Limpieza automática de servicios y datos de prueba
- **Reports**: Reportes HTML detallados con screenshots y videos
- **CI/CD Ready**: Configurado para pipelines de CI/CD

## 📁 Estructura del Proyecto

```
apicentric/
├── webui/
│   ├── tests/
│   │   ├── e2e/                      # Tests E2E principales
│   │   │   ├── 01-basic-navigation.spec.ts
│   │   │   ├── 02-dashboard.spec.ts
│   │   │   ├── 03-service-management.spec.ts
│   │   │   ├── 04-backend-integration.spec.ts
│   │   │   └── global.setup.ts       # Setup global
│   │   ├── fixtures/                 # Datos de prueba
│   │   │   └── test-data.ts
│   │   └── utils/                    # Utilidades de testing
│   │       ├── api-helper.ts         # Helper para APIs backend
│   │       └── webui-helper.ts       # Helper para UI frontend
│   ├── playwright.config.ts          # Configuración Playwright
│   └── package.json                  # Scripts y dependencias
└── testing/
    ├── run-e2e-tests.sh              # Script principal de ejecución
    └── README.md                     # Esta documentación
```

## 🛠 Instalación y Configuración

### Prerrequisitos

- **Node.js** >= 18.0.0
- **npm** >= 8.0.0
- **Rust** >= 1.70.0
- **Cargo** (incluido con Rust)
- **curl** y **lsof** (para health checks)

### Instalación

1. **Instalar dependencias del frontend:**
   ```bash
   cd webui
   npm install
   ```

2. **Instalar navegadores de Playwright:**
   ```bash
   npx playwright install
   ```

3. **Compilar el backend:**
   ```bash
   cd .. # volver al root del proyecto
   cargo build --release
   ```

### Verificación de Instalación

```bash
# Verificar que el script es ejecutable
ls -la testing/run-e2e-tests.sh

# Mostrar ayuda del script
./testing/run-e2e-tests.sh --help
```

## 🧪 Ejecución de Tests

### Ejecución Completa (Recomendada)

Ejecuta backend, frontend y todos los tests:

```bash
./testing/run-e2e-tests.sh
```

### Opciones de Ejecución

```bash
# Tests con navegador visible (debugging)
./testing/run-e2e-tests.sh --headed

# Tests en Firefox
./testing/run-e2e-tests.sh --browser firefox

# Solo ejecutar tests (servicios ya running)
./testing/run-e2e-tests.sh --tests-only

# Verbose output
./testing/run-e2e-tests.sh --verbose

# Mantener servicios corriendo después de tests
./testing/run-e2e-tests.sh --no-cleanup
```

### Ejecución Manual

Si prefieres control manual:

```bash
# 1. Iniciar backend
cargo run --release -- cloud --port 8080 &

# 2. Iniciar frontend
cd webui
npm run dev &

# 3. Ejecutar tests
npm run test:e2e
```

### Scripts NPM Disponibles

En el directorio `webui/`:

```bash
npm run test:e2e          # Tests headless
npm run test:e2e:headed   # Tests con navegador visible
npm run test:e2e:ui       # Playwright UI mode
npm run test:e2e:debug    # Debug mode
npm run test:e2e:report   # Mostrar último reporte
```

## 📊 Tipos de Tests

### 1. Basic Navigation Tests (`01-basic-navigation.spec.ts`)
- ✅ Carga de la aplicación
- ✅ Navegación entre vistas
- ✅ Sidebar navigation
- ✅ Responsive behavior
- ✅ State management

### 2. Dashboard Tests (`02-dashboard.spec.ts`)
- ✅ Display de servicios
- ✅ Service cards
- ✅ Simulator status
- ✅ Real-time updates
- ✅ Start/Stop controls

### 3. Service Management Tests (`03-service-management.spec.ts`)
- ✅ Listado de servicios
- ✅ Creación de servicios
- ✅ Start/Stop servicios
- ✅ Service status
- ✅ Actions menu
- ✅ Delete confirmation

### 4. Backend Integration Tests (`04-backend-integration.spec.ts`)
- ✅ Health checks
- ✅ Simulator start/stop
- ✅ Service CRUD operations
- ✅ YAML validation
- ✅ Request logs
- ✅ Concurrent operations
- ✅ Error handling

## ⚙️ Configuración

### Playwright Configuration (`playwright.config.ts`)

```typescript
export default defineConfig({
  testDir: './tests/e2e',
  timeout: 30 * 1000,
  use: {
    baseURL: 'http://localhost:9002',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    { name: 'chromium' },
    { name: 'firefox' },
    { name: 'webkit' },
  ],
})
```

### Environment Variables

```bash
# URLs de servicios
BACKEND_URL=http://localhost:8080
FRONTEND_URL=http://localhost:9002
NEXT_PUBLIC_API_URL=http://localhost:8080

# Configuración de tests
CI=true                    # Para ejecución en CI
HEADLESS=true             # Headless mode
BROWSER=chromium          # Browser por defecto
```

### Data Test IDs

Los elementos de UI están etiquetados con `data-testid` para facilitar testing:

```html
<!-- Sidebar navigation -->
<button data-testid="sidebar-dashboard">Dashboard</button>
<button data-testid="sidebar-services">Services</button>

<!-- Simulator control -->
<button data-testid="simulator-toggle">Start Simulator</button>

<!-- Service cards -->
<div data-testid="service-card" data-service-name="my-service">
  <span data-testid="service-status">running</span>
  <button data-testid="start-service-button">Start</button>
  <button data-testid="stop-service-button">Stop</button>
</div>

<!-- Service management -->
<button data-testid="create-service-button">Create Service</button>
```

## 🔧 Desarrollo de Tests

### Page Object Model

Utilizamos el patrón Page Object Model con helpers:

```typescript
// WebUI Helper - Interacciones con frontend
const webUI = new WebUIHelper(page);
await webUI.navigateToDashboard();
await webUI.createService('my-service', yamlContent);

// API Helper - Interacciones con backend
const apiHelper = new ApiTestHelper();
const status = await apiHelper.getSimulatorStatus();
await apiHelper.startService('my-service');
```

### Fixtures y Test Data

```typescript
import { SAMPLE_SERVICE_YAML, TEST_SCENARIOS } from '../fixtures/test-data';

// Usar datos de prueba predefinidos
const service = TEST_SCENARIOS.basicCrud;
await apiHelper.createService(service.yaml);
```

### Best Practices

1. **Cleanup**: Siempre limpiar datos de prueba
2. **Waits**: Usar `expect().toBeVisible()` en lugar de `waitForTimeout()`
3. **Isolation**: Tests independientes entre sí
4. **Error Handling**: Manejar errores esperados gracefully
5. **Screenshots**: Tomar screenshots en puntos clave para debugging

### Ejemplo de Test

```typescript
test('should create and manage service', async ({ page }) => {
  const webUI = new WebUIHelper(page);
  const apiHelper = new ApiTestHelper();
  const serviceName = `test-${Date.now()}`;
  
  try {
    // Crear servicio vía API
    await apiHelper.createService(serviceYaml);
    
    // Verificar en UI
    await webUI.navigateToServices();
    await expect(page.getByTestId(`service-${serviceName}`)).toBeVisible();
    
    // Start service vía UI
    await webUI.startService(serviceName);
    
    // Verificar estado
    await expect(page.getByTestId('service-status')).toContainText('running');
    
  } finally {
    // Cleanup
    await apiHelper.deleteService(serviceName);
  }
});
```

## 🏗 CI/CD Integration

### GitHub Actions

```yaml
name: E2E Tests
on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install dependencies
        run: |
          cd webui && npm install
          npx playwright install --with-deps
          
      - name: Run E2E tests
        run: ./testing/run-e2e-tests.sh --verbose
        
      - name: Upload test results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: webui/playwright-report/
```

### Docker Support

```dockerfile
FROM mcr.microsoft.com/playwright:v1.40.0-focal

# Copiar proyecto
COPY . /app
WORKDIR /app

# Instalar dependencias
RUN cd webui && npm install

# Ejecutar tests
CMD ["./testing/run-e2e-tests.sh", "--verbose"]
```

## 🐛 Troubleshooting

### Problemas Comunes

#### Tests fallan con "Backend not available"
```bash
# Verificar que el backend compile
cargo build --release

# Verificar puertos disponibles
lsof -i :8080
lsof -i :9002

# Ejecutar con verbose
./testing/run-e2e-tests.sh --verbose
```

#### Frontend no inicia
```bash
# Limpiar node_modules
cd webui
rm -rf node_modules package-lock.json
npm install

# Verificar versión de Node
node --version  # Debe ser >= 18
```

#### Tests intermitentes
```bash
# Ejecutar en modo headed para debugging
./testing/run-e2e-tests.sh --headed --browser chromium

# Revisar screenshots y videos
ls webui/test-results/
```

#### Permisos en macOS
```bash
# Hacer ejecutable el script
chmod +x testing/run-e2e-tests.sh

# Dar permisos a Playwright para controlar navegadores
# Ir a System Preferences > Security & Privacy > Accessibility
```

### Logs y Debugging

```bash
# Logs del backend
tail -f /tmp/apicentric-backend.log

# Logs del frontend
tail -f /tmp/apicentric-frontend.log

# Test results
open webui/playwright-report/index.html

# Ver traces de tests fallidos
npx playwright show-trace webui/test-results/test-trace.zip
```

### Performance Tuning

```bash
# Reducir workers para máquinas lentas
npx playwright test --workers=1

# Ejecutar solo tests específicos
npx playwright test 01-basic-navigation

# Skip setup para iteración rápida
npx playwright test --project=chromium
```

## 📈 Métricas y Reportes

### Test Reports

Después de ejecutar tests, los reportes están disponibles en:

- **HTML Report**: `webui/playwright-report/index.html`
- **JSON Results**: `webui/test-results/results.json`
- **JUnit XML**: `webui/test-results/junit.xml`

### Coverage Integration

Para agregar coverage de código:

```javascript
// playwright.config.ts
use: {
  // ...existing config
  trace: 'retain-on-failure',
  screenshot: 'only-on-failure',
  video: 'retain-on-failure',
}
```

## 🤝 Contribuir

### Añadir Nuevos Tests

1. Crear archivo en `webui/tests/e2e/`
2. Seguir convención de nombres: `XX-feature-name.spec.ts`
3. Usar helpers existentes (`WebUIHelper`, `ApiTestHelper`)
4. Agregar data-testids necesarios al frontend
5. Incluir cleanup en `afterEach`

### Mejoras al Framework

- Extender helpers con nuevas funcionalidades
- Agregar fixtures para nuevos escenarios
- Mejorar error handling y reporting
- Optimizar performance de tests

---

## 📞 Soporte

Para problemas específicos del sistema de testing:

1. Revisar logs en `/tmp/apicentric-*.log`
2. Ejecutar con `--verbose` para más información
3. Usar `--headed` para debugging visual
4. Consultar traces en `test-results/`

**Happy Testing! 🧪✨**