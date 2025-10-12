// Librería pública para lh-traffic
pub mod app;

// ============================================================================
// MACROS GLOBALES - Disponibles en todo el proyecto
// ============================================================================
// Estas macros están disponibles globalmente usando #[macro_export]
// Se pueden usar en cualquier parte del proyecto sin importar módulos

/// Macro para logging rápido con colores (nivel INFO)
///
/// # Ejemplos
/// ```
/// log_info!("Servidor iniciado");
/// log_info!("Usuario {} conectado", username);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("\x1b[32m[INFO]\x1b[0m {}", format!($($arg)*));
    };
}

/// Macro para logging de errores (nivel ERROR)
///
/// # Ejemplos
/// ```
/// log_error!("Falló la conexión");
/// log_error!("Error al conectar a {}", db_host);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("\x1b[31m[ERROR]\x1b[0m {}", format!($($arg)*));
    };
}

/// Macro para logging de advertencias (nivel WARN)
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        println!("\x1b[33m[WARN]\x1b[0m {}", format!($($arg)*));
    };
}

/// Macro para logging de debug (nivel DEBUG)
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        println!("\x1b[36m[DEBUG]\x1b[0m {}", format!($($arg)*));
    };
}

/// Macro para medir tiempo de ejecución de un bloque de código
///
/// # Ejemplos
/// ```
/// measure_time!("get_connections", {
///     let connections = service.get_all().await?;
///     process(connections);
/// });
/// ```
#[macro_export]
macro_rules! measure_time {
    ($nombre:expr, $bloque:block) => {
        {
            use std::time::Instant;
            let __inicio = Instant::now();

            let __resultado = $bloque;

            let __duracion = __inicio.elapsed();
            println!("\x1b[35m[TIEMPO]\x1b[0m {} tomó {:?}", $nombre, __duracion);

            __resultado
        }
    };
}

/// Macro para crear HashMap fácilmente (similar a vec![])
///
/// # Ejemplos
/// ```
/// let map = hashmap!{
///     "nombre" => "Juan",
///     "edad" => "25",
/// };
/// ```
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $val:expr),* $(,)?) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(
                map.insert($key, $val);
            )*
            map
        }
    };
}

// ============================================================================
// PRELUDE - Importaciones comunes para facilitar el uso
// ============================================================================
// Usa "use lh_traffic::prelude::*;" al inicio de tus archivos para tener
// acceso a todos los tipos y macros comunes sin imports individuales

/// Módulo prelude con importaciones comunes
pub mod prelude {
    // Re-exportar macros globales (ya están disponibles, pero así quedan explícitas)
    pub use crate::{log_info, log_error, log_warn, log_debug, measure_time, hashmap};

    // Re-exportar tipos comunes del módulo networking
    pub use crate::app::modules::networking::{
        GetLocalNetworkTrafficService,
        GetLocalNetworkTrafficInputDto,
        GotLocalNetworkTrafficDto,
    };

    // Re-exportar componentes compartidos comunes
    pub use crate::app::modules::shared::infrastructure::components::cli::CliColor;
    pub use crate::app::modules::shared::infrastructure::components::logger::Logger;

    // Re-exportar tipos estándar de Rust que se usan frecuentemente
    pub use std::collections::HashMap;
    pub use anyhow::{Result, Context};
}
