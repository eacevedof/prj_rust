# Módulos Disponibles para Migración

Este documento lista todos los módulos del proyecto Deno original que están disponibles para migrar a Rust.

## Estado de Migración

- ✅ **Shared** - Completado (infraestructura base)
- ⏳ Pendiente de migración (elije cuál migrar)

## Módulos Disponibles

### 1. Authenticator
**Descripción**: Módulo de autenticación
- Validación de tokens de aplicación
- Validación de tokens de dispositivo
- Gestión de autenticación

### 2. Devops
**Descripción**: Operaciones DevOps
- Health checks
- Migraciones de base de datos
- Verificaciones de sistema

### 3. Documentation
**Descripción**: Documentación de la API
- Vista de documentación
- Changelog
- Routes de documentación

### 4. Domains
**Descripción**: Gestión de dominios
- CRUD de dominios
- Operaciones batch
- Consultas de dominios

### 5. ETL
**Descripción**: Procesos de Extract, Transform, Load
- ETL de dominios riesgosos
- Refresh de Redis
- Procesamiento batch

### 6. Elastic
**Descripción**: Integración con Elasticsearch
- Logging a Elasticsearch
- Escritura de documentos
- Queries y búsquedas

### 7. Events
**Descripción**: Sistema de eventos
- Event bus
- Event handlers
- Event publishing

### 8. Firebase
**Descripción**: Integración con Firebase
- Push notifications (FCM)
- Autenticación OAuth2 con Google
- ⚠️ **NOTA**: Contiene credenciales sensibles que NO deben publicarse

### 9. HealthCheck
**Descripción**: Endpoints de health check
- Status del servicio
- Verificación de conexiones
- Métricas de salud

### 10. Mailings
**Descripción**: Sistema de correos
- Envío de emails
- Templates
- Configuración SMTP

### 11. Notifications
**Descripción**: Sistema de notificaciones
- Notificaciones push
- Notificaciones en app
- Gestión de suscripciones

### 12. Phishing
**Descripción**: **MÓDULO PRINCIPAL** - Detección de phishing
- Evaluación de riesgo de dominios
- Scoring de peligrosidad
- Cache de resultados
- API endpoints principales

### 13. Projects
**Descripción**: Gestión de proyectos
- CRUD de proyectos
- Asociación de dominios
- Configuración por proyecto

### 14. StaticAssets
**Descripción**: Archivos estáticos
- Servir assets
- Gestión de recursos
- Rutas públicas

### 15. UserDevices
**Descripción**: Dispositivos de usuario
- Registro de dispositivos
- Tokens de dispositivo
- Push notification tokens

### 16. Users
**Descripción**: Gestión de usuarios
- CRUD de usuarios
- Autenticación
- Permisos y roles

## Recomendación de Orden de Migración

Según la arquitectura del proyecto (microservicio de anti-phishing), sugiero este orden:

1. **HealthCheck** - Simple y útil para testing
2. **Phishing** - Módulo principal del negocio
3. **Domains** - Necesario para Phishing
4. **ETL** - Para procesamiento de datos
5. **Authenticator** - Seguridad de la API
6. **Notifications/Firebase** - Para alertas
7. **Los demás según necesidad**

## ¿Cuál módulo quieres migrar primero?
