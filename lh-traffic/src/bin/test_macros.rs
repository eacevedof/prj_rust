/// Binario de ejemplo para probar las macros globales
/// Ejecutar con: cargo run --bin test-macros

// ============================================================================
// FORMA 1: Usar el prelude (importa todo lo común)
// ============================================================================
use lh_traffic::prelude::*;

#[tokio::main]
async fn main() {
    println!("\n=== PROBANDO MACROS GLOBALES ===\n");

    // ========================================================================
    // 1. MACROS DE LOGGING (disponibles globalmente)
    // ========================================================================
    println!("1. Macros de logging:\n");

    log_info!("Aplicación iniciada correctamente");
    log_warn!("Advertencia: Puerto 8080 ya en uso");
    log_error!("Error al conectar a la base de datos");
    log_debug!("Debug: Variable x = {}", 42);

    println!();

    // ========================================================================
    // 2. MACRO PARA MEDIR TIEMPO
    // ========================================================================
    println!("2. Macro measure_time:\n");

    let resultado = measure_time!("operación_lenta", {
        // Simular operación lenta
        std::thread::sleep(std::time::Duration::from_millis(100));
        "Operación completada"
    });

    log_info!("Resultado: {}", resultado);

    println!();

    // ========================================================================
    // 3. MACRO HASHMAP (similar a vec![])
    // ========================================================================
    println!("3. Macro hashmap:\n");

    let usuario = hashmap! {
        "nombre".to_string() => "Juan".to_string(),
        "edad".to_string() => "25".to_string(),
        "email".to_string() => "juan@example.com".to_string(),
    };

    log_info!("Usuario creado: {:?}", usuario);

    println!();

    // ========================================================================
    // 4. USANDO TIPOS DEL PRELUDE (sin imports adicionales)
    // ========================================================================
    println!("4. Tipos del prelude (sin imports):\n");

    // HashMap está disponible sin import porque está en el prelude
    let mut datos: HashMap<String, i32> = HashMap::new();
    datos.insert("conexiones".to_string(), 42);

    log_info!("Datos: {:?}", datos);

    // Result está disponible sin import
    let operacion: Result<i32> = Ok(100);
    match operacion {
        Ok(val) => log_info!("Operación exitosa: {}", val),
        Err(e) => log_error!("Error: {}", e),
    }

    println!();

    // ========================================================================
    // 5. USANDO SERVICIOS DEL PRELUDE
    // ========================================================================
    println!("5. Servicios del prelude:\n");

    // GetLocalNetworkTrafficService está disponible sin import
    let service = GetLocalNetworkTrafficService::get_instance();
    let input = GetLocalNetworkTrafficInputDto::empty();

    match service.invoke(input).await {
        Ok(output) => {
            log_info!("Encontradas {} conexiones de red", output.total);
        }
        Err(e) => {
            log_error!("Error al obtener conexiones: {}", e);
        }
    }

    println!();

    // ========================================================================
    // 6. MACROS ANIDADAS
    // ========================================================================
    println!("6. Macros anidadas:\n");

    measure_time!("proceso_completo", {
        log_info!("Iniciando proceso...");

        let data = hashmap! {
            "paso1".to_string() => "completado".to_string(),
            "paso2".to_string() => "completado".to_string(),
        };

        log_info!("Proceso: {:?}", data);

        "Proceso exitoso"
    });

    println!("\n=== FIN DE PRUEBAS ===\n");
}
