# Apicentric Cloud ☁️
[No implementado]
Plan:

## Visión General

**Apicentric Cloud** es la versión nativa en la nube de Apicentric, diseñada con arquitectura hexagonal, vanguardista y superperformante para crear, probar y simular APIs a escala empresarial.

### 🌟 Características Principales

- **🏗️ Arquitectura Hexagonal**: Código limpio, testeable y mantenible
- **⚡ Superperformante**: Construido con Rust y tecnologías de vanguardia
- **🌐 Cloud Native**: Diseñado para escalabilidad y alta disponibilidad
- **🔄 API REST completa**: Toda la funcionalidad accesible vía HTTP
- **🖥️ Interfaz Web moderna**: Frontend React reutilizado y adaptado
- **🐳 Containerización**: Listo para Docker y Kubernetes
- **🔧 Fácil despliegue**: Scripts automatizados para múltiples entornos

## 🚀 Inicio Rápido

### Desarrollo Local

```bash
# Clonar el repositorio
git clone https://github.com/pmaojo/apicentric.git
cd apicentric

# Ejecutar en modo desarrollo
./deploy.sh dev
```

### Despliegue con Docker

```bash
# Construir imagen
./deploy.sh build

# Desplegar en producción
./deploy.sh deploy

# Ver logs
./deploy.sh logs

# Realizar pruebas de salud
./deploy.sh test
```

## 📚 Arquitectura

### Componentes Principales

1. **Servidor Cloud** (`src/cloud/`): Servidor HTTP con Axum
2. **API REST** (`src/cloud/api/`): Endpoints para todas las funcionalidades
3. **Frontend Web** (`gui/`): Interfaz React adaptada de Tauri
4. **Core Engine** (`src/simulator/`): Motor de simulación reutilizado

### Endpoints Principales

- `GET /health` - Health check
- `GET /api/services` - Lista servicios activos
- `POST /api/services/load` - Cargar definición de servicio
- `POST /api/services/save` - Guardar definición de servicio

## 🛠️ Scripts de Despliegue

### Comandos Disponibles

```bash
./deploy.sh dev      # Desarrollo local
./deploy.sh build    # Construir imagen Docker
./deploy.sh deploy   # Desplegar producción
./deploy.sh stop     # Detener servicios
./deploy.sh logs     # Ver logs
./deploy.sh test     # Pruebas de salud
./deploy.sh clean    # Limpieza completa
./deploy.sh help     # Ayuda
```

## 🌐 Configuración de Producción

### Variables de Entorno

```env
APICENTRIC_ENV=production
APICENTRIC_LOG_LEVEL=info
RUST_LOG=apicentric=info
```

### Docker Compose

El archivo `docker-compose.yml` incluye:
- Servidor Apicentric Cloud
- Proxy Nginx (opcional, perfil production)
- Volúmenes persistentes
- Health checks automáticos

### Nginx (Opcional)

Para entornos de producción, se incluye configuración Nginx con:
- Proxy reverso
- Headers de seguridad
- Caché de assets estáticos
- Soporte HTTPS/SSL

## 🔧 Desarrollo

### Estructura del Proyecto

```
src/
├── cloud/           # Módulo cloud
│   ├── server.rs    # Servidor HTTP principal
│   ├── handlers.rs  # Handlers de API
│   └── api.rs       # DTOs y estructuras
├── bin/
│   ├── apicentric.rs       # CLI original
│   └── apicentric-cloud.rs # Servidor cloud
└── ...             # Resto de módulos

packaging/
├── Dockerfile      # Imagen Docker optimizada
└── ...

gui/                # Frontend React (en migración)
deploy.sh          # Script de despliegue
docker-compose.yml # Orquestación de servicios
nginx.conf        # Configuración proxy
```

### Compilación

```bash
# Servidor cloud
cargo build --bin apicentric-cloud

# CLI tradicional
cargo build --bin apicentric

# Todos los binarios
cargo build --release
```

## 🎯 Migración desde CLI/Tauri

La migración incluye:

1. ✅ **Renombrado completo**: `apicentric`/`apicentric` → `apicentric`
2. ✅ **Servidor HTTP**: Nueva arquitectura cloud-native
3. ✅ **API REST**: Exposición de toda la funcionalidad
4. 🔄 **Frontend**: Adaptación React (en progreso)
5. 📦 **Containerización**: Docker y orquestación completa

## 🚢 Despliegue en la Nube

### Providers Recomendados

- **DigitalOcean App Platform**
- **AWS ECS/Fargate**
- **Google Cloud Run**
- **Azure Container Instances**
- **Render.com**

### Kubernetes

```yaml
# Ejemplo de deployment básico
apiVersion: apps/v1
kind: Deployment
metadata:
  name: apicentric-cloud
spec:
  replicas: 3
  selector:
    matchLabels:
      app: apicentric-cloud
  template:
    metadata:
      labels:
        app: apicentric-cloud
    spec:
      containers:
      - name: apicentric-cloud
        image: apicentric-cloud:latest
        ports:
        - containerPort: 8080
        env:
        - name: APICENTRIC_ENV
          value: "production"
```

## 🤝 Contribuir

1. Fork el proyecto
2. Crear branch para feature (`git checkout -b feature/AmazingFeature`)
3. Commit cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push al branch (`git push origin feature/AmazingFeature`)
5. Abrir Pull Request

## 📄 Licencia

Distribuido bajo la licencia MIT. Ver `LICENSE` para más información.

## 🙋‍♂️ Soporte

- **GitHub Issues**: [Reportar bugs](https://github.com/pmaojo/apicentric/issues)
- **Documentación**: [Wiki del proyecto](https://github.com/pmaojo/apicentric/wiki)
- **Discusiones**: [GitHub Discussions](https://github.com/pmaojo/apicentric/discussions)

---

**Apicentric Cloud** - Donde la simulación de APIs encuentra la nube ☁️✨
