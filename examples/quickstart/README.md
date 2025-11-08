# Quick Start Example

Este directorio contiene un ejemplo completo de cómo usar Apicentric para simular una API de tareas.

## Archivos

- `tasks-api.yaml` - Definición del servicio API
- `test-requests.http` - Peticiones de ejemplo para probar la API
- `generated/` - Archivos generados por Apicentric

## Pasos

1. **Validar el servicio**:
   ```bash
   apicentric simulator validate --path tasks-api.yaml --verbose
   ```

2. **Iniciar el simulador**:
   ```bash
   apicentric simulator start --services-dir . --force
   ```

3. **Probar endpoints** (en otra terminal):
   ```bash
   # Listar tareas
   curl http://localhost:9001/api/v1/tasks
   
   # Crear tarea
   curl -X POST http://localhost:9001/api/v1/tasks \
     -H "Content-Type: application/json" \
     -d '{"title": "Nueva tarea", "description": "Ejemplo de tarea"}'
   
   # Obtener tarea específica
   curl http://localhost:9001/api/v1/tasks/1
   ```

4. **Generar tipos TypeScript**:
   ```bash
   apicentric simulator export-types --input tasks-api.yaml --output generated/types.ts
   ```

5. **Generar hooks React Query**:
   ```bash
   apicentric simulator export-query --input tasks-api.yaml --output generated/hooks.ts
   ```

## Estructura del servicio

El archivo `tasks-api.yaml` define:
- 📝 Operaciones CRUD para tareas
- 🎭 Respuestas dinámicas con Handlebars
- 🔄 Escenarios para testing (modo lento, errores)
- 📊 Fixtures con datos de ejemplo
- 🌐 Configuración CORS y logging