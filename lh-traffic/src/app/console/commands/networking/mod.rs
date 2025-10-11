// ============================================================================
// MOD.RS - Archivo de módulo en Rust
// ============================================================================
//
// ¿POR QUÉ EXISTE ESTE ARCHIVO?
// ------------------------------
// En Rust, cada CARPETA que quieras usar como módulo DEBE tener un archivo
// llamado "mod.rs". Este archivo le dice a Rust:
// 1. Qué submódulos (archivos .rs) existen dentro de esta carpeta
// 2. Qué elementos (structs, funciones, enums) queremos exportar públicamente
//
// EQUIVALENCIA EN OTROS LENGUAJES:
// ---------------------------------
// - PHP: Similar a un archivo index.php que importa clases con "require"
// - TypeScript: Similar a un archivo index.ts que hace "export { ... }"
// - Python: Similar a __init__.py en un paquete
//
// SIN ESTE ARCHIVO:
// -----------------
// Si no existiera este mod.rs, Rust NO sabría que la carpeta "networking"
// es un módulo y daría error de compilación.
//
// ============================================================================

// SECCIÓN 1: DECLARACIÓN DE MÓDULOS (obligatorio)
// ============================================================================
// Aquí le decimos a Rust: "estos archivos .rs son parte de este módulo"
// Si no declaras un módulo aquí, NO podrás usarlo en tu código

pub mod list_network_connections_command;   // Declara: list_network_connections_command.rs
pub mod watch_network_connections_command;  // Declara: watch_network_connections_command.rs

// NOTA: El nombre del módulo es el nombre del archivo SIN la extensión .rs
//       Si tuvieras un archivo "foo_bar.rs", lo declararías como: pub mod foo_bar;


// SECCIÓN 2: RE-EXPORTACIONES (opcional pero recomendado)
// ============================================================================
// Estas líneas hacen que los structs sean más fáciles de usar desde fuera
//
// SIN re-exportación, para usar ListNetworkConnectionsCommand tendrías que escribir:
//   use crate::app::console::commands::networking::list_network_connections_command::ListNetworkConnectionsCommand;
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MUY LARGO!
//
// CON re-exportación (pub use), puedes escribir simplemente:
//   use crate::app::console::commands::networking::ListNetworkConnectionsCommand;
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MUCHO MÁS CORTO!

pub use list_network_connections_command::ListNetworkConnectionsCommand;
pub use watch_network_connections_command::WatchNetworkConnectionsCommand;

// RESUMEN COMPARATIVO:
// ====================
// Archivo físico:           list_network_connections_command.rs
// Módulo declarado como:    list_network_connections_command
// Struct dentro del archivo: ListNetworkConnectionsCommand
//
// Sin pub use → Ruta larga:  networking::list_network_connections_command::ListNetworkConnectionsCommand
// Con pub use → Ruta corta:  networking::ListNetworkConnectionsCommand
//
// ============================================================================
