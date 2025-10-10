# Configuración del Workspace para WSL2

Este proyecto está configurado para ejecutarse en WSL2 desde VS Code en Windows.

## Prerrequisitos

1. **WSL2 instalado y configurado**:
   ```powershell
   wsl --install
   ```

2. **Rust instalado en WSL2**:
   ```bash
   # Ejecutar dentro de WSL2
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **Extensiones de VS Code recomendadas**:
   - `rust-lang.rust-analyzer` - Soporte para Rust
   - `ms-vscode-remote.remote-wsl` - Desarrollo en WSL

## Cómo ejecutar la aplicación

### Opción 1: Usar el botón de Play (▶️)
1. Presiona `Ctrl+Shift+P` y busca "Run Task"
2. Selecciona `cargo run main (WSL)` (esta es la tarea por defecto)
3. O usa `Ctrl+Shift+P` → "Tasks: Run Build Task" para ejecutar la tarea principal

### Opción 2: Usar configuraciones de depuración
1. Ve a la pestaña "Run and Debug" (`Ctrl+Shift+D`)
2. Selecciona una de las configuraciones disponibles:
   - `Run main (WSL)` - Ejecuta el binario principal
   - `Run network_traffic (WSL)` - Ejecuta el binario network_traffic
   - `Run list-network-connections (WSL)` - Lista conexiones de red
   - `Run watch-network-connections (WSL)` - Monitorea conexiones

### Opción 3: Tareas disponibles
- `cargo build (WSL)` - Compilar el proyecto
- `cargo run main (WSL)` - Ejecutar binario principal ⭐ (tarea por defecto)
- `cargo run network_traffic (WSL)` - Ejecutar monitor de tráfico
- `cargo run list-network-connections (WSL)` - Listar conexiones
- `cargo run watch-network-connections (WSL)` - Monitorear conexiones
- `cargo check (WSL)` - Verificar código sin compilar
- `cargo test (WSL)` - Ejecutar pruebas

## Estructura del proyecto

```
src/
├── main.rs                          # Punto de entrada principal
├── lib.rs                          # Biblioteca principal
├── bin/                            # Binarios ejecutables
│   ├── network_traffic.rs          # Monitor de tráfico de red
│   ├── list_network_connections.rs # Listar conexiones
│   └── watch_network_connections.rs # Monitorear conexiones
└── modules/                        # Módulos de la aplicación
```

## Binarios disponibles

1. **main** (`cargo run`): Aplicación principal con comandos
2. **network_traffic**: Monitor de tráfico de red
3. **list-network-connections**: Lista conexiones actuales
4. **watch-network-connections**: Monitorea conexiones en tiempo real

## Comandos útiles

### En terminal WSL (Terminal → New Terminal → WSL):
```bash
# Navegar al proyecto
cd /mnt/c/projects/prj-rust/lh-traffic

# Compilar
cargo build

# Ejecutar diferentes binarios
cargo run
cargo run --bin network_traffic
cargo run --bin list-network-connections
cargo run --bin watch-network-connections

# Verificar código
cargo check

# Ejecutar tests
cargo test
```

## Troubleshooting

### Si aparece "cargo: command not found":
Este error ocurre cuando Rust está instalado pero el entorno no se carga correctamente. **Ya está solucionado** en este workspace usando `bash -l -c` que carga el perfil completo.

Si el problema persiste:
```bash
# Verificar que Rust esté instalado en WSL
wsl bash -l -c "cargo --version"

# Si no está instalado, instalarlo:
wsl bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

# Luego cargar el entorno:
wsl bash -c "source ~/.cargo/env && cargo --version"
```

### Si WSL no responde:
```powershell
# Reiniciar WSL
wsl --shutdown
wsl
```

### Si el proyecto no compila:
```bash
# Limpiar y recompilar
cd /mnt/c/projects/prj-rust/lh-traffic
cargo clean
cargo build
```

### Verificar la configuración:
```bash
# Verificar que WSL puede ejecutar cargo
wsl bash -l -c "cd /mnt/c/projects/prj-rust/lh-traffic && cargo check"
```

## Configuración actual

- **Tarea por defecto**: `cargo run main (WSL)`
- **Directorio WSL**: `/mnt/c/projects/prj-rust/lh-traffic`
- **Terminal**: PowerShell con perfil WSL disponible
- **Rust Analyzer**: Configurado para usar WSL

Para cambiar la tarea por defecto, edita `.vscode/tasks.json` y modifica el campo `"isDefault": true`.