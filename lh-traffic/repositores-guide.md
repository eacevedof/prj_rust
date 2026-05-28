# Network Repositories Guide

Este proyecto incluye 6 repositorios principales para obtener información de red:

## 1. SystemNetworkReaderRepository

Obtiene las conexiones de red activas del sistema usando comandos nativos del SO.

### Plataformas soportadas:

- ✅ **Windows**: netstat -ano
- ✅ **Linux**: ss -antp

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
    pub program_name: Option<String>, // "PID:58608" (temporal, se resuelve después)
}
```

---

## 2. ProcessInfoRepository

Obtiene información detallada de un proceso a partir de su PID.

### Plataformas soportadas:

- ✅ **Windows**: tasklist.exe (compatible con Cygwin)
- ✅ **Linux**: /proc filesystem

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::ProcessInfoRepository;

let repo = ProcessInfoRepository::new();

// Obtener información completa del proceso
let info = repo.get_process_info(1234).await?;

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
    pub path: String,                    // "Unknown" (Windows) o path completo (Linux)
    pub working_directory: Option<String>, // None
    pub owner: Option<String>,           // None (Windows via tasklist)
}
```

### Implementación Windows:

**Problema resuelto**: PowerShell desde Cygwin no funcionaba correctamente.

**Solución**: Usar tasklist.exe directamente:

```rust
Command::new("tasklist.exe")
    .args(&["/FI", &format!("PID eq {}", pid)])
    .output()
```

**Parser**: Skip 3 líneas (vacía + header + separador) y extraer `parts[0]`:

```rust
for line in output.lines().skip(3) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let name = parts[0].to_string(); // "mullvadbrowser.exe"
}
```

---

## 3. WhoisRepository

Obtiene información de IPs usando el comando `whois` (rápido, sin límite de rate).

### Características:

- ✅ **Sin límite de rate**: Usa comando local whois
- ✅ **Rápido**: ~100-300ms por consulta
- ✅ **Organización**: Obtiene netname/org-name
- ⚠️ **No siempre tiene país**: Algunos resultados no incluyen country

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::WhoisRepository;

let repo = WhoisRepository::new();

// Obtener información whois
let info = repo.get_whois_info("8.8.8.8").await?;

// Obtener solo la organización
let org = repo.get_organization("8.8.8.8").await?; // "Google LLC"
```

### Estructura de retorno:

```rust
pub struct WhoisInfo {
    pub ip: String,                   // "8.8.8.8"
    pub country: Option<String>,      // "US" (si está disponible)
    pub organization: Option<String>, // "Google LLC"
    pub asn: Option<String>,          // "AS15169"
    pub ip_range: Option<String>,     // "8.8.8.0/24"
}
```

---

## 4. IpGeolocationRepository

Obtiene información de geolocalización usando ip-api.com (cuando whois no tiene país).

### Características:

- ✅ **Geolocalización precisa**: País, ciudad, ISP
- ⚠️ **Rate limit**: 45 requests/minuto (versión gratuita)
- ✅ **Filtra IPs locales**: No consulta IPs privadas
- ⚠️ **Solo HTTP**: HTTPS requiere versión Pro

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::IpGeolocationRepository;

let repo = IpGeolocationRepository::new();

// Obtener información completa
let info = repo.get_geolocation("8.8.8.8").await?;

// Obtener solo el país
let country = repo.get_country("8.8.8.8").await?; // "United States"
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

---

## 5. HybridIpInfoRepository

**Repositorio inteligente** que combina whois + ip-api.com + caché para obtener la mejor información.

### Estrategia:

1. **Verificar caché primero** (15 días de TTL)
2. Si no está en caché:
   - a. Consultar **whois** primero (rápido, sin límite)
   - b. Si whois tiene país → usar whois, listo ✅
   - c. Si whois NO tiene país → consultar **ip-api.com**
   - d. Combinar la mejor información de ambos
   - e. Guardar en caché

### Ventajas:

- ✅ **Sin límite de rate**: Usa API solo cuando es necesario
- ✅ **Caché de 15 días**: Evita consultas repetidas
- ✅ **Información completa**: Organización + geolocalización
- ✅ **Optimizado para velocidad**: whois primero, API solo si es necesario

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::HybridIpInfoRepository;

let repo = HybridIpInfoRepository::new();

// Obtener información híbrida (best of both worlds)
let info = repo.get_ip_info("8.8.8.8").await?;

// Obtener solo el país (usa caché si existe)
let country = repo.get_country("8.8.8.8").await?; // "United States"

// Obtener solo la organización
let org = repo.get_organization("8.8.8.8").await?; // "Google LLC"

// Extraer IP de "IP:Puerto"
let ip = repo.extract_ip_from_address("8.8.8.8:443"); // "8.8.8.8"
```

### Estructura de retorno:

```rust
pub struct HybridIpInfo {
    pub ip: String,                   // "8.8.8.8"
    pub country: String,              // "United States" (preferencia: api > whois)
    pub country_code: Option<String>, // "US" (solo de api)
    pub city: Option<String>,         // "Mountain View" (solo de api)
    pub organization: String,         // "Google LLC" (preferencia: whois > api.isp)
    pub isp: Option<String>,          // "Google LLC" (solo de api)
    pub asn: Option<String>,          // "AS15169" (solo de whois)
    pub ip_range: Option<String>,     // "8.8.8.0/24" (solo de whois)
    pub source: String,               // "whois", "api", "hybrid", "cache"
}
```

---

## 6. IpCacheRepository

Sistema de caché JSON con TTL de 15 días para evitar consultas repetidas.

### Características:

- ✅ **Un archivo JSON por IP**: `storage/cache/8.8.8.8.json`
- ✅ **TTL de 15 días**: Expira automáticamente
- ✅ **Auto-cleanup**: Elimina entradas expiradas
- ✅ **IPv6 compatible**: Reemplaza `:` por `_` en el nombre del archivo

### Uso básico:

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::IpCacheRepository;

let repo = IpCacheRepository::new();

// Obtener de caché (retorna None si no existe o expiró)
let cached = repo.get("8.8.8.8").await?;

// Guardar en caché
let info = HybridIpInfo { /* ... */ };
repo.set("8.8.8.8", info).await?;

// Verificar si existe (sin verificar expiración)
let exists = repo.exists("8.8.8.8").await;

// Eliminar de caché
repo.delete("8.8.8.8").await?;

// Limpiar todas las entradas expiradas
let cleaned_count = repo.clean_expired().await?;

// Obtener estadísticas del caché
let size_bytes = repo.get_cache_size().await?;
let count = repo.get_cache_count().await?;
```

### Estructura del archivo JSON:

```json
{
  "ip": "8.8.8.8",
  "data": {
    "ip": "8.8.8.8",
    "country": "United States",
    "country_code": "US",
    "city": "Mountain View",
    "organization": "Google LLC",
    "isp": "Google LLC",
    "asn": "AS15169",
    "ip_range": "8.8.8.0/24",
    "source": "hybrid"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "expires_at": "2024-01-30T10:30:00Z"
}
```

---

## Ejemplo completo: Conexiones enriquecidas

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    HybridIpInfoRepository,
    ProcessInfoRepository,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let network_repo = SystemNetworkReaderRepository::new();
    let hybrid_repo = HybridIpInfoRepository::new();
    let process_repo = ProcessInfoRepository::new();

    // 1. Obtener conexiones
    let connections = network_repo.get_established_connections().await?;

    for conn in connections.iter() {
        println!("Connection: {} -> {}", conn.local_address, conn.foreign_address);

        // 2. Obtener información de la IP remota (con caché automático)
        let remote_ip = hybrid_repo.extract_ip_from_address(&conn.foreign_address);
        if let Some(info) = hybrid_repo.get_ip_info(&remote_ip).await? {
            println!("  Country: {} ({})", info.country, info.country_code.unwrap_or_default());
            if let Some(city) = info.city {
                println!("  City: {}", city);
            }
            println!("  Organization: {}", info.organization);
            println!("  Source: {}", info.source); // "cache", "whois", "api", "hybrid"
        }

        // 3. Obtener nombre real del proceso
        if let Some(pid) = conn.pid {
            if let Ok(Some(name)) = process_repo.get_process_name(pid).await {
                println!("  Process: {}:{}", pid, name);
            }
        }

        println!();

        // Rate limiting solo para IPs remotas (200ms)
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    Ok(())
}
```

---

## Filtros avanzados

### Lógica AND con filtros múltiples:

```bash
# Filtro simple
make run-list-filter f=established

# Filtro AND: established AND remote
make run-list-filter f="established,remote"

# Múltiples filtros AND
make run-list-filter f="tcp,established,remote"
```

### Filtro especial "remote":

Excluye IPs locales automáticamente:
- `127.*` (loopback)
- `192.168.*` (red privada)
- `0.0.0.0` (wildcard)
- `*` (any)

```rust
// Implementación del filtro "remote"
if filter_lower == "remote" {
    let remote_ip = hybrid_repo.extract_ip_from_address(&conn.foreign_address);
    return !remote_ip.starts_with("127.")
        && !remote_ip.starts_with("192.168.")
        && !remote_ip.starts_with("0.0.0.0")
        && remote_ip != "*";
}
```

---

## Notas de rendimiento

1. **SystemNetworkReaderRepository**: Rápido (< 100ms)
2. **ProcessInfoRepository**:
   - Windows (tasklist): ~50-100ms por proceso
   - Linux (/proc): ~5-10ms por proceso
3. **WhoisRepository**: Medio (~100-300ms por IP)
4. **IpGeolocationRepository**: Lento (~1-2s por IP) + rate limit 45/min
5. **HybridIpInfoRepository**: Inteligente (usa cache → whois → api)
6. **IpCacheRepository**: Muy rápido (~1-5ms)

### Recomendaciones:

- ✅ **Primera ejecución**: Lenta (consulta whois/api para cada IP)
- ✅ **Ejecuciones posteriores**: Muy rápida (usa caché de 15 días)
- ✅ **Rate limiting**: 200ms entre conexiones remotas (automático)
- ✅ **IPs locales**: No consultan API (sin rate limit)

---

## Comandos disponibles en Makefile

```bash
# Ver todas las conexiones
make run-list

# Ver conexiones filtradas
make run-list-filter f=established
make run-list-filter f="established,remote"
make run-list-filter f=firefox
```

---

## Troubleshooting

### Windows: Process names no aparecen (muestra "PID:58608")

**Causa**: Parser de tasklist estaba leyendo el separador "===" como nombre.

**Solución aplicada**:
```rust
// Skip 3 líneas: vacía + header + separador
for line in output.lines().skip(3) {
    if line.is_empty() || line.starts_with("=") {
        continue;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    let name = parts[0].to_string(); // Ahora funciona correctamente
}
```

### Error: "ip-api.com returned error status"

**Causa**: Excediste el rate limit (45 req/min).

**Solución**: El proyecto ya incluye:
- Caché automático de 15 días
- Rate limiting de 200ms entre requests
- Filtro automático de IPs locales

### Cache no funciona

**Verificar**:
```bash
# Ver archivos de caché
ls -lh storage/cache/

# Ver contenido de un archivo
cat storage/cache/8.8.8.8.json

# Limpiar caché manualmente
rm -rf storage/cache/*.json
```

---

## TODO / Mejoras futuras

- [x] ~~Implementar caché de IPs~~ ✅ (15 días JSON)
- [x] ~~Soporte para Windows~~ ✅ (tasklist + netstat)
- [x] ~~Filtros AND~~ ✅ (comma-separated)
- [x] ~~Combinar whois + API~~ ✅ (HybridIpInfoRepository)
- [ ] Agregar soporte para IPv6 completo
- [ ] Implementar `clean_expired()` automático al inicio
- [ ] Agregar comando para ver estadísticas del caché
- [ ] Tests unitarios para todos los repositorios
