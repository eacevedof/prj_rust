# Network Repositories Guide

Este proyecto incluye 3 repositorios principales para obtener información de red:

## 1. SystemNetworkReaderRepository

Obtiene las conexiones de red activas del sistema.

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::SystemNetworkReaderRepository;

let repo = SystemNetworkReaderRepository::new();

// Obtener todas las conexiones
let connections = repo.get_local_network_traffic().await?;

// Filtrar solo conexiones establecidas
let established = repo.get_established_connections().await?;

// Filtrar por puerto local
let port_80_connections = repo.get_connections_by_local_port(80).await?;
```

### Estructura de retorno:

```rust
pub struct NetworkConnectionEntity {
    pub protocol: String,         // "TCP", "UDP"
    pub local_address: String,    // "192.168.1.100:45678"
    pub foreign_address: String,  // "93.184.216.34:443"
    pub state: String,            // "ESTABLISHED", "LISTEN", etc.
    pub pid: Option<u32>,         // Process ID
    pub program_name: Option<String>, // "firefox.exe"
}
```

---

## 2. IpGeolocationRepository

Obtiene información de geolocalización para IPs remotas usando el servicio gratuito de ip-api.com.

### Características:

- ✅ Servicio gratuito (45 requests/minuto)
- ✅ Filtra automáticamente IPs locales/privadas
- ✅ Retorna país, ciudad, ISP, coordenadas
- ⚠️ Solo HTTP (no HTTPS en versión gratuita)

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::IpGeolocationRepository;

let repo = IpGeolocationRepository::new();

// Obtener información completa
let info = repo.get_geolocation("8.8.8.8").await?;

// Obtener solo el código de país
let country_code = repo.get_country_code("8.8.8.8").await?; // "US"

// Obtener solo el país
let country = repo.get_country("8.8.8.8").await?; // "United States"

// Extraer IP de un address "IP:Puerto"
let ip = repo.extract_ip_from_address("93.184.216.34:443"); // "93.184.216.34"
```

### Estructura de retorno:

```rust
pub struct IpGeolocationInfo {
    pub query: String,         // "8.8.8.8"
    pub country: String,       // "United States"
    pub country_code: String,  // "US"
    pub region: String,        // "California"
    pub city: String,          // "Mountain View"
    pub isp: String,           // "Google LLC"
    pub org: String,           // "Google Public DNS"
    pub lat: f64,              // 37.4056
    pub lon: f64,              // -122.0775
}
```

### Rate Limiting:

⚠️ **IMPORTANTE**: El servicio gratuito tiene límite de **45 requests por minuto**.

Para evitar excederlo:

```rust
// Agregar pausas entre requests
for ip in ips.iter() {
    if let Some(info) = repo.get_geolocation(ip).await? {
        println!("{} - {}", ip, info.country);
    }

    // Pausa de 1.5 segundos entre requests
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
}
```

### Alternativas para producción:

1. **Implementar caché con Redis**:
```rust
// Ejemplo de flujo con caché
let redis_key = format!("geo:{}", ip);

// Intentar obtener de caché
if let Some(cached) = redis.get(&redis_key).await? {
    return Ok(Some(cached));
}

// Si no está en caché, consultar API
let info = repo.get_geolocation(ip).await?;

// Guardar en caché por 7 días
redis.set_with_ttl(&redis_key, &info, 7 * 24 * 60).await?;
```

2. **Usar versión Pro de ip-api.com**:
   - HTTPS incluido
   - Sin límite de rate
   - $13/mes para 150k requests

3. **Usar MaxMind GeoIP2**:
   - Base de datos local (sin requests HTTP)
   - Más rápido
   - Requires actualización mensual

---

## 3. ProcessInfoRepository

Obtiene información detallada de un proceso a partir de su PID.

### Plataformas soportadas:

- ✅ **Windows**: Usa PowerShell + WMI
- ✅ **Linux**: Usa `/proc` filesystem

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::ProcessInfoRepository;

let repo = ProcessInfoRepository::new();

// Obtener información completa del proceso
let info = repo.get_process_info(1234).await?;

// Obtener solo el path
let path = repo.get_process_path(1234).await?; // "C:\Program Files\Firefox\firefox.exe"

// Obtener solo el nombre
let name = repo.get_process_name(1234).await?; // "firefox.exe"

// Verificar si un proceso existe
let exists = repo.process_exists(1234).await; // true/false
```

### Estructura de retorno:

```rust
pub struct ProcessInfo {
    pub pid: u32,                        // 1234
    pub name: String,                    // "firefox.exe"
    pub path: String,                    // "C:\Program Files\Firefox\firefox.exe"
    pub working_directory: Option<String>, // None (no implementado por ahora)
    pub owner: Option<String>,           // "DESKTOP\User"
}
```

### Diferencias entre plataformas:

| Campo | Windows | Linux |
|-------|---------|-------|
| `name` | ✅ Nombre con extensión | ✅ Nombre sin extensión |
| `path` | ✅ Path completo | ✅ Symlink de /proc/{pid}/exe |
| `owner` | ✅ DOMAIN\User | ⚠️ Solo UID |

### Permisos:

En **Windows**, necesitas privilegios para consultar procesos de otros usuarios. Si no tienes permisos:
- El comando PowerShell fallará silenciosamente
- Se retornará `None`

En **Linux**, necesitas:
- Leer `/proc/{pid}/` requiere permisos del proceso
- Algunos procesos del kernel no tienen executable path

---

## Ejemplo completo: Conexiones enriquecidas

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    IpGeolocationRepository,
    ProcessInfoRepository,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let network_repo = SystemNetworkReaderRepository::new();
    let geo_repo = IpGeolocationRepository::new();
    let process_repo = ProcessInfoRepository::new();

    // 1. Obtener conexiones
    let connections = network_repo.get_established_connections().await?;

    for conn in connections.iter().take(10) {
        println!("Connection: {} -> {}", conn.local_address, conn.foreign_address);

        // 2. Obtener país de la IP remota
        let remote_ip = geo_repo.extract_ip_from_address(&conn.foreign_address);
        if let Some(geo) = geo_repo.get_geolocation(&remote_ip).await? {
            println!("  Country: {} ({})", geo.country, geo.country_code);
            println!("  City: {}", geo.city);
            println!("  ISP: {}", geo.isp);
        }

        // 3. Obtener información del proceso
        if let Some(pid) = conn.pid {
            if let Some(process) = process_repo.get_process_info(pid).await? {
                println!("  Process: {} (PID: {})", process.name, process.pid);
                println!("  Path: {}", process.path);
                if let Some(owner) = process.owner {
                    println!("  Owner: {}", owner);
                }
            }
        }

        println!();

        // Rate limiting para geolocalización
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    }

    Ok(())
}
```

---

## Comandos disponibles en Makefile

```bash
# Ver todas las conexiones
make run-list

# Ver conexiones filtradas
make run-list-filter f=firefox

# Modo watch (auto-refresh)
make run-watch

# Ver información enriquecida (con geolocalización y proceso)
make run-enhanced
```

---

## Notas de rendimiento

1. **SystemNetworkReaderRepository**: Rápido (< 100ms)
2. **IpGeolocationRepository**: Lento (1-2s por IP) - Usar con rate limiting
3. **ProcessInfoRepository**:
   - Windows: Medio (~200-500ms por proceso) - PowerShell es lento
   - Linux: Rápido (~10-50ms por proceso) - Solo lee archivos

### Recomendaciones:

- ✅ Usar caché de Redis para geolocalización
- ✅ Hacer requests de geolocalización en paralelo (respetando rate limit)
- ✅ En Windows, considerar usar WMI directamente sin PowerShell para mejor performance
- ✅ Procesar solo IPs públicas (filtrar locales/privadas)

---

## Testing

```bash
# Compilar
cargo build

# Ejecutar ejemplo básico
cargo run --bin list-network-connections

# Ejecutar ejemplo enriquecido
cargo run --bin enhanced-network-info

# Ejecutar con makefile
make run-enhanced
```

---

## Troubleshooting

### Error: "Failed to execute PowerShell command"

**Causa**: PowerShell no está en el PATH o no tienes permisos.

**Solución**:
- Verificar que PowerShell está instalado
- Ejecutar como Administrador
- En Linux, este error no debería aparecer (usa /proc)

### Error: "ip-api.com returned error status"

**Causa**: Excediste el rate limit (45 req/min).

**Solución**:
- Agregar pausas entre requests (1.5s mínimo)
- Implementar caché local
- Considerar versión Pro de la API

### Geolocalización retorna None para todas las IPs

**Causa**: Todas las IPs son locales/privadas.

**Solución**:
- Verificar que hay conexiones a IPs públicas
- Usar `conn.foreign_address` no `conn.local_address`
- Revisar la salida de `netstat` o `ss`

---

## TODO / Mejoras futuras

- [ ] Implementar caché de Redis para geolocalización
- [ ] Agregar soporte para bases de datos offline (MaxMind)
- [ ] Mejorar performance de ProcessInfoRepository en Windows (usar WMI directo)
- [ ] Agregar método `get_working_directory()` en ProcessInfo
- [ ] Agregar soporte para IPv6 en geolocalización
- [ ] Agregar tests unitarios
- [ ] Agregar documentación de API con rustdoc
