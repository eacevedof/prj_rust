# Módulo Networking

Módulo para monitorear y analizar el tráfico de red local, siguiendo la arquitectura DDD del proyecto original en Deno.

## Estructura

```
src/modules/networking/
├── domain/
│   ├── types/
│   │   └── network_connection.rs    # Tipo NetworkConnection
│   └── enums/
│       └── connection_state.rs      # Estados de conexión
├── application/
│   └── get_local_network_traffic_service.rs  # Caso de uso principal
└── infrastructure/
    └── repositories/
        └── system_network_reader_repository.rs  # Lectura del sistema
```

## Uso del Servicio

### Ejemplo básico

```rust
use lh_traffic::modules::networking::GetLocalNetworkTrafficService;

#[tokio::main]
async fn main() {
    let service = GetLocalNetworkTrafficService::new();

    // Obtener todas las conexiones
    let connections = service.invoke().await.unwrap();
    println!("Total connections: {}", connections.len());

    // Obtener resumen
    let summary = service.get_summary().await.unwrap();
    summary.print();

    // Solo conexiones establecidas
    let established = service.get_established_only().await.unwrap();
    println!("Established: {}", established.len());

    // Conexiones por puerto
    let port_80 = service.get_by_port(80).await.unwrap();
    println!("Port 80 connections: {}", port_80.len());
}
```

### Ejecutar el binario de ejemplo

```bash
# Compilar y ejecutar
cargo run --bin network_traffic

# Solo compilar
cargo build --bin network_traffic
```

## NetworkConnection

```rust
pub struct NetworkConnection {
    pub protocol: String,          // "tcp", "udp"
    pub local_address: String,      // "192.168.1.100:45678"
    pub foreign_address: String,    // "93.184.216.34:443"
    pub state: String,              // "ESTABLISHED", "LISTEN", etc.
    pub pid: Option<u32>,           // PID del proceso
    pub program_name: Option<String>, // Nombre del programa
}
```

## Métodos del Servicio

### `invoke() -> Result<Vec<NetworkConnection>>`
Obtiene todas las conexiones de red activas en el sistema.

### `get_established_only() -> Result<Vec<NetworkConnection>>`
Filtra solo las conexiones en estado ESTABLISHED.

### `get_by_port(port: u16) -> Result<Vec<NetworkConnection>>`
Obtiene conexiones que usan un puerto local específico.

### `get_summary() -> Result<NetworkTrafficSummary>`
Genera un resumen estadístico del tráfico:
- Total de conexiones
- Conexiones TCP/UDP
- Conexiones establecidas
- Sockets en escucha

## Arquitectura DDD

### Domain Layer
- **Types**: Entidades del dominio (`NetworkConnection`)
- **Enums**: Valores de dominio (`ConnectionState`)

### Application Layer
- **Services**: Casos de uso del negocio
  - `GetLocalNetworkTrafficService`: Orquesta la obtención de tráfico de red

### Infrastructure Layer
- **Repositories**: Acceso a recursos del sistema
  - `SystemNetworkReaderRepository`: Lee información de red vía comando `ss`

## Requisitos del Sistema

El módulo usa el comando `ss` (Socket Statistics) que está disponible en sistemas Linux modernos.

### Instalar ss (si no está disponible)
```bash
# Ubuntu/Debian
sudo apt-get install iproute2

# CentOS/RHEL
sudo yum install iproute

# Arch
sudo pacman -S iproute2
```

## Testing

```bash
# Ejecutar tests del módulo
cargo test --lib networking

# Test específico
cargo test --lib system_network_reader_repository::tests
```

## Equivalencia con el proyecto Deno

| Deno (TypeScript) | Rust |
|-------------------|------|
| `Devops/Application/Services/CheckApp/CheckAppService.ts` | `networking/application/get_local_network_traffic_service.rs` |
| `Devops/Infrastructure/Repositories/SystemOsReaderRepository.ts` | `networking/infrastructure/repositories/system_network_reader_repository.rs` |
| `PsAuxType` (TypeScript) | `NetworkConnection` (Rust) |

## Próximas mejoras

- [ ] Soporte para Windows (netstat)
- [ ] Soporte para macOS (lsof)
- [ ] Caché de resultados
- [ ] Filtros avanzados (por IP, protocolo, etc.)
- [ ] Serialización JSON para API REST
- [ ] Métricas en tiempo real
