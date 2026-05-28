# Módulo Networking

Módulo para monitorear y analizar el tráfico de red local con geolocalización de IPs y descubrimiento de procesos, siguiendo la arquitectura DDD del proyecto original en Deno.

## Estructura

```
src/modules/networking/
├── domain/
│   └── entities/
│       └── network_connection_entity.rs    # Entidad NetworkConnectionEntity
├── application/
│   └── commands/
│       └── list_network_connections_command.rs  # Comando CLI principal
└── infrastructure/
    └── repositories/
        ├── system_network_reader_repository.rs   # netstat/ss parser
        ├── process_info_repository.rs            # tasklist/proc reader
        ├── whois_repository.rs                   # whois command wrapper
        ├── ip_geolocation_repository.rs          # ip-api.com client
        ├── hybrid_ip_info_repository.rs          # whois + api hybrid
        └── ip_cache_repository.rs                # 15-day JSON cache
```

## Casos de uso

### 1. Listar todas las conexiones

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::SystemNetworkReaderRepository;

#[tokio::main]
async fn main() {
    let repo = SystemNetworkReaderRepository::new();

    // Obtener todas las conexiones
    let connections = repo.get_local_network_traffic().await.unwrap();
    println!("Total connections: {}", connections.len());
}
```

### 2. Conexiones establecidas con geolocalización

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    HybridIpInfoRepository,
};

#[tokio::main]
async fn main() {
    let network_repo = SystemNetworkReaderRepository::new();
    let ip_repo = HybridIpInfoRepository::new();

    // Obtener conexiones establecidas
    let connections = network_repo.get_established_connections().await.unwrap();

    for conn in connections.iter() {
        // Obtener país de la IP remota (usa caché si existe)
        let remote_ip = ip_repo.extract_ip_from_address(&conn.foreign_address);
        if let Ok(Some(info)) = ip_repo.get_ip_info(&remote_ip).await {
            println!("{} -> {} ({}, {})",
                conn.local_address,
                conn.foreign_address,
                info.country,
                info.organization
            );
        }
    }
}
```

### 3. Descubrir proceso de cada conexión

```rust
use lh_traffic::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    ProcessInfoRepository,
};

#[tokio::main]
async fn main() {
    let network_repo = SystemNetworkReaderRepository::new();
    let process_repo = ProcessInfoRepository::new();

    let connections = network_repo.get_established_connections().await.unwrap();

    for conn in connections.iter() {
        if let Some(pid) = conn.pid {
            // Obtener nombre real del proceso
            if let Ok(Some(name)) = process_repo.get_process_name(pid).await {
                println!("{}:{} -> {}",
                    pid,
                    name,
                    conn.foreign_address
                );
            }
        }
    }
}
```

## Entidades principales

### NetworkConnectionEntity

```rust
pub struct NetworkConnectionEntity {
    pub protocol: String,           // "TCP", "UDP"
    pub local_address: String,      // "192.168.1.100:45678"
    pub foreign_address: String,    // "93.184.216.34:443"
    pub state: String,              // "ESTABLISHED", "LISTEN", etc.
    pub pid: Option<u32>,           // PID del proceso
    pub program_name: Option<String>, // Nombre temporal (se resuelve después)
}
```

### HybridIpInfo

```rust
pub struct HybridIpInfo {
    pub ip: String,                   // "8.8.8.8"
    pub country: String,              // "United States"
    pub country_code: Option<String>, // "US"
    pub city: Option<String>,         // "Mountain View"
    pub organization: String,         // "Google LLC"
    pub isp: Option<String>,          // "Google LLC"
    pub asn: Option<String>,          // "AS15169"
    pub ip_range: Option<String>,     // "8.8.8.0/24"
    pub source: String,               // "cache", "whois", "api", "hybrid"
}
```

### ProcessInfo

```rust
pub struct ProcessInfo {
    pub pid: u32,                        // 58608
    pub name: String,                    // "mullvadbrowser.exe"
    pub path: String,                    // Path del ejecutable (Windows: "Unknown")
    pub working_directory: Option<String>, // None
    pub owner: Option<String>,           // None (Windows via tasklist)
}
```

## Arquitectura DDD

### Domain Layer
- **Entities**: Entidades del dominio (`NetworkConnectionEntity`)
- **Value Objects**: IPs, puertos, estados

### Application Layer
- **Commands**: Comandos CLI
  - `ListNetworkConnectionsCommand`: Lista conexiones con info enriquecida

### Infrastructure Layer
- **Repositories**: Acceso a recursos del sistema y externos
  - `SystemNetworkReaderRepository`: Lee netstat/ss del sistema
  - `ProcessInfoRepository`: Lee tasklist/proc del sistema
  - `WhoisRepository`: Ejecuta comando whois
  - `IpGeolocationRepository`: Consulta API de ip-api.com
  - `HybridIpInfoRepository`: Combina whois + API + caché
  - `IpCacheRepository`: Persistencia JSON con TTL

## Sistema de caché

### Ubicación y formato

```
storage/cache/
├── 8.8.8.8.json           # Google DNS
├── 142.251.209.246.json   # Google server
└── 2001_4860_4860__8888.json  # IPv6 (: reemplazado por _)
```

### Estructura del caché

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
  "created_at": "2024-01-15T10:30:00.000Z",
  "expires_at": "2024-01-30T10:30:00.000Z"
}
```

### TTL: 15 días

- Creación: Al consultar una IP por primera vez
- Expiración: 15 días después de `created_at`
- Auto-cleanup: Al leer una entrada expirada, se elimina automáticamente

## Estrategia híbrida de consulta de IPs

### Flujo de HybridIpInfoRepository:

1. **Check caché** (instantáneo)
   - Si existe y no expiró → devolver ✅
   - Si no existe o expiró → continuar al paso 2

2. **Consultar whois** (~100-300ms)
   - Sin límite de rate
   - Proporciona: organización, ASN, rango de IPs
   - A veces incluye país ✅

3. **Decidir si consultar API**
   - ✅ Si whois tiene país → listo, usar whois
   - ⚠️ Si whois NO tiene país → continuar al paso 4

4. **Consultar ip-api.com** (~1-2s)
   - Rate limit: 45 requests/min
   - Proporciona: país, ciudad, ISP, coordenadas

5. **Combinar resultados**
   - País: preferencia API (más preciso)
   - Organización: preferencia whois (más detallado)
   - Ciudad, ISP: solo de API
   - ASN, rango: solo de whois

6. **Guardar en caché** (15 días)

### Ventajas de este enfoque:

- ✅ **Sin límite efectivo**: Usa API solo cuando whois no tiene país
- ✅ **Rápido**: whois es rápido y sin límite
- ✅ **Completo**: Combina lo mejor de ambas fuentes
- ✅ **Eficiente**: Caché de 15 días evita consultas repetidas

## Requisitos del Sistema

### Windows

```powershell
# Comandos nativos (ya incluidos en Windows)
tasklist.exe
netstat.exe

# Comando externo (instalar manualmente)
whois.exe  # Descargar de Sysinternals
```

### Linux

```bash
# Ubuntu/Debian
sudo apt-get install iproute2 whois

# CentOS/RHEL
sudo yum install iproute whois

# Arch
sudo pacman -S iproute2 whois
```

## Filtros avanzados

### Sintaxis de filtros:

```bash
# Filtro simple (busca en todos los campos)
make run-list-filter f=firefox

# Filtro AND (múltiples condiciones)
make run-list-filter f="established,remote"

# Filtro especial "remote" (excluye IPs locales)
make run-list-filter f=remote
```

### Lógica de filtros:

```rust
// Todos los filtros deben coincidir (AND)
filters.iter().all(|filter_lower| {
    // Filtro especial: "remote"
    if filter_lower == "remote" {
        let remote_ip = extract_ip_from_address(&conn.foreign_address);
        return !is_local_ip(&remote_ip);
    }

    // Filtro general: busca en todos los campos (OR)
    conn.protocol.to_lowercase().contains(filter_lower)
        || conn.state.to_lowercase().contains(filter_lower)
        || conn.foreign_address.to_lowercase().contains(filter_lower)
        || conn.program_name.as_ref().map_or(false, |p| p.to_lowercase().contains(filter_lower))
})
```

## Testing

```bash
# Compilar
cargo build --release

# Ejecutar comando básico
make run-list

# Ejecutar con filtros
make run-list-filter f=established
make run-list-filter f="established,remote"

# Ver cache
ls -lh storage/cache/
```

## Equivalencia con el proyecto Deno

| Deno (TypeScript) | Rust |
|-------------------|------|
| `Devops/Application/Services/CheckApp/CheckAppService.ts` | `networking/application/commands/list_network_connections_command.rs` |
| `Devops/Infrastructure/Repositories/SystemOsReaderRepository.ts` | `networking/infrastructure/repositories/system_network_reader_repository.rs` |
| `PsAuxType` (TypeScript) | `NetworkConnectionEntity` (Rust) |

## Mejoras implementadas vs proyecto Deno

- [x] ✅ **Soporte para Windows** (tasklist + netstat)
- [x] ✅ **Caché de IPs** (15 días JSON)
- [x] ✅ **Filtros avanzados** (AND logic)
- [x] ✅ **Información de IPs** (whois + API + cache)
- [x] ✅ **Descubrimiento de procesos** (tasklist fixed)
- [x] ✅ **Colores en output** (ESTABLISHED=green, LISTEN=yellow)

## Próximas mejoras

- [ ] Comando para limpiar caché expirado manualmente
- [ ] Comando para ver estadísticas del caché
- [ ] Soporte para IPv6 completo
- [ ] Modo watch (auto-refresh cada N segundos)
- [ ] Export a JSON/CSV
- [ ] Tests unitarios completos
- [ ] Documentación con rustdoc

## Bug fixes aplicados

### 1. Process names mostraban "PID:58608" en lugar del nombre real

**Problema**: El parser de tasklist leía el separador "===" como nombre de proceso.

**Causa**: `skip(2)` solo saltaba 2 líneas, pero tasklist tiene:
```
Line 0: (vacía)
Line 1: Header
Line 2: Separador ===    ← Parser leía esto como nombre!
Line 3: Datos reales
```

**Solución**:
```rust
// Cambio de skip(2) a skip(3)
for line in output.lines().skip(3) {
    if line.is_empty() || line.starts_with("=") {
        continue;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    let name = parts[0].to_string(); // Ahora funciona ✅
}
```

### 2. Prioridad incorrecta en resolución de nombres

**Problema**: Usaba `conn.program_name` ("PID:58608") sin intentar resolver el nombre real.

**Solución**: Invertir prioridad:
```rust
// ANTES (incorrecto)
let name = if let Some(ref n) = conn.program_name {
    n.clone()  // Usaba "PID:58608" directamente
} else if let Ok(Some(info)) = process_repo.get_process_name(p).await {
    info
}

// DESPUÉS (correcto)
let name = if let Ok(Some(info)) = process_repo.get_process_name(p).await {
    info  // PRIMERO intenta resolver nombre real
} else if let Some(ref n) = conn.program_name {
    n.clone()  // Fallback
}
```

---

**Última actualización**: 2024-01-15
