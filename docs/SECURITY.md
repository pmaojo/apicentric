# Seguridad del Proyecto ApiCentric

## Resumen Ejecutivo

Este documento describe las medidas de seguridad implementadas en ApiCentric, incluyendo auditorías de vulnerabilidades, prácticas de codificación segura y recomendaciones para el mantenimiento continuo de la seguridad.

## Estado de Vulnerabilidades

### Auditoría de Dependencias (cargo audit)

**Última auditoría:** 2025-11-13

#### Vulnerabilidades Encontradas
- **1 vulnerabilidad crítica**: `serde_yaml` v0.7.5 - Recursión incontrolada en deserialización
  - **ID:** RUSTSEC-2018-0005
  - **Solución:** Actualizar a `serde_yaml` >= 0.8.4
  - **Estado:** Pendiente - Se planea migrar a `openapiv3` crate en la próxima versión para resolver esta vulnerabilidad
  - **Recomendación:** Migrar a `openapiv3` crate como alternativa más moderna

#### Advertencias de Mantenimiento
- `adler` v1.0.2 - Sin mantenimiento (reemplazar con `adler2`)
- `fxhash` v0.2.1 - Sin mantenimiento
- `paste` v1.0.15 - Sin mantenimiento
- `yaml-rust` v0.4.5 - Sin mantenimiento

**Estado general:** 1 vulnerabilidad pendiente, 4 advertencias de mantenimiento.

## Mejoras de Seguridad Implementadas

### 1. Autenticación y Autorización

#### Hashing de Contraseñas
- **Algoritmo:** Argon2 (recomendado por OWASP)
- **Implementación:** Sal aleatoria generada con `OsRng`
- **Ubicación:** `src/auth/password.rs`

#### Tokens JWT
- **Firma:** HMAC con clave secreta configurable
- **Expiración:** TTL configurable (horas)
- **Validación:** Verificación de firma y expiración
- **Ubicación:** `src/auth/jwt.rs`

#### Blacklist de Tokens
- **Propósito:** Revocación de tokens antes de expiración (logout)
- **Implementación:** HashSet thread-safe con hashing para eficiencia
- **Ubicación:** `src/auth/blacklist.rs`

#### Middleware de Autenticación
- **Validación:** Bearer tokens requeridos
- **Verificación:** Firma, expiración y blacklist
- **Errores:** Códigos específicos para diferentes tipos de fallo
- **Ubicación:** `src/auth/middleware.rs`

### 2. Configuración CORS

#### Modo Desarrollo
- **Política:** Permisiva (todos los orígenes)
- **Propósito:** Facilita desarrollo local

#### Modo Producción
- **Política:** Restrictiva con orígenes específicos
- **Configuración:** Variable `ALLOWED_ORIGINS`
- **Fallback:** localhost si no configurado
- **Headers:** Solo Authorization, Content-Type, Accept
- **Credentials:** Habilitadas con max-age de 1 hora
- **Ubicación:** `src/cloud/cors.rs`

### 3. Logging Seguro

#### Características
- **Estructurado:** Uso de `tracing` con campos contextuales
- **Configurable:** Niveles por módulo vía `RUST_LOG`
- **Formato:** JSON para producción, pretty para desarrollo
- **Performance:** Macros lazy para debug logging
- **Ubicación:** `src/logging.rs`

### 4. Configuración de Compilación

#### Perfil Release
- **Optimización:** LTO habilitado, codegen-units=1
- **Tamaño:** Strip de símbolos, panic=abort
- **Seguridad:** Reduce superficie de ataque eliminando unwinding
- **Ubicación:** `Cargo.toml`

### 5. Dependencias Seguras

#### Criptografía
- `argon2`: Para hashing de contraseñas
- `jsonwebtoken`: Para tokens JWT
- `rustls-tls`: TLS moderno (en reqwest)

#### HTTP
- `axum`: Framework web moderno con async/await
- `hyper`: HTTP/2 soportado
- `tower-http`: Middlewares seguros

## Estándares de Seguridad Cumplidos

### OWASP Top 10
- **A02:2021 - Cryptographic Failures**: Uso de Argon2 y HMAC
- **A03:2021 - Injection**: Validación de entrada, uso de prepared statements en SQLite
- **A05:2021 - Security Misconfiguration**: CORS restrictivo en producción
- **A07:2021 - Identification and Authentication Failures**: JWT con expiración y blacklist

### Mejores Prácticas Rust
- **Memory Safety**: Rust garantiza seguridad de memoria
- **Type Safety**: Sistema de tipos fuerte previene errores comunes
- **Error Handling**: Uso de `Result` y `thiserror` para manejo robusto
- **Async Safety**: Tokio con runtime seguro

## Recomendaciones para Mantenimiento

### Auditorías Periódicas
- Ejecutar `cargo audit` semanalmente
- Revisar dependencias mensualmente con `cargo outdated`
- Monitorear avisos de seguridad de crates.io

### Actualizaciones de Dependencias
- Mantener versiones actualizadas, especialmente crates de seguridad
- Usar `cargo update` con precaución
- Probar thoroughly después de actualizaciones

### Configuración en Producción
- Establecer `APICENTRIC_ENV=production`
- Configurar `ALLOWED_ORIGINS` apropiadamente
- Usar `RUST_LOG` para logging estructurado
- Configurar `APICENTRIC_LOG_FORMAT=json`

### Monitoreo
- Logs de autenticación fallida
- Alertas en intentos de acceso no autorizado
- Monitoreo de dependencias vulnerables

## Contacto

Para reportes de seguridad, por favor contactar al equipo de desarrollo a través de GitHub Issues con la etiqueta "security".

## Historial de Cambios

- **2025-11-13**: Auditoría de seguridad ejecutada, estado sin cambios. Se planea migrar a openapiv3 para resolver vulnerabilidades pendientes.
- Actualización del documento de seguridad