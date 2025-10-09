/// Binario de ejemplo para probar el módulo Networking con DTOs
/// Ejecutar con: cargo run --bin network_traffic

use lh_traffic::modules::networking::{
    GetLocalNetworkTrafficService,
    GetLocalNetworkTrafficInputDto,
};

#[tokio::main]
async fn main() {
    // Inicializar tracing
    tracing_subscriber::fmt::init();

    println!("=== LH Traffic - Network Monitor ===\n");

    // Crear el servicio
    let service = GetLocalNetworkTrafficService::new();

    // Ejemplo 1: Obtener todas las conexiones (sin filtro)
    println!("1. Obteniendo TODAS las conexiones...\n");
    let input = GetLocalNetworkTrafficInputDto::empty();

    match service.invoke(input).await {
        Ok(output) => {
            println!("Total: {} conexiones\n", output.total);

            // Mostrar las primeras 10 filas
            for (i, row) in output.rows.iter().enumerate().take(10) {
                println!("Conexión #{}:", i + 1);
                println!("  Protocol: {}", row.get("protocol").unwrap_or(&"-".to_string()));
                println!("  Local: {}", row.get("local_address").unwrap_or(&"-".to_string()));
                println!("  Remote: {}", row.get("foreign_address").unwrap_or(&"-".to_string()));
                println!("  State: {}", row.get("state").unwrap_or(&"-".to_string()));
                println!("  PID: {}", row.get("pid").unwrap_or(&"-".to_string()));
                println!("  Program: {}", row.get("program_name").unwrap_or(&"-".to_string()));
                println!();
            }

            if output.total > 10 {
                println!("... y {} conexiones más\n", output.total - 10);
            }

            // Serializar a JSON
            println!("JSON de las primeras 3 filas:");
            let first_three: Vec<_> = output.rows.iter().take(3).cloned().collect();
            println!("{}\n", serde_json::to_string_pretty(&first_three).unwrap_or_default());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    println!("═══════════════════════════════════\n");

    // Ejemplo 2: Filtrar por programa específico
    println!("2. Filtrando por 'firefox'...\n");
    let input_filtered = GetLocalNetworkTrafficInputDto::new("firefox".to_string());

    match service.invoke(input_filtered).await {
        Ok(output) => {
            println!("Encontradas {} conexiones de Firefox\n", output.total);

            for (i, row) in output.rows.iter().enumerate() {
                println!(
                    "  {}. {} -> {} | State: {} | PID: {}",
                    i + 1,
                    row.get("local_address").unwrap_or(&"-".to_string()),
                    row.get("foreign_address").unwrap_or(&"-".to_string()),
                    row.get("state").unwrap_or(&"-".to_string()),
                    row.get("pid").unwrap_or(&"-".to_string()),
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n═══════════════════════════════════\n");

    // Ejemplo 3: Filtrar por puerto
    println!("3. Filtrando por puerto ':80'...\n");
    let input_port = GetLocalNetworkTrafficInputDto::new(":80".to_string());

    match service.invoke(input_port).await {
        Ok(output) => {
            println!("Encontradas {} conexiones en puerto 80\n", output.total);

            for (i, row) in output.rows.iter().enumerate() {
                println!(
                    "  {}. {} <-> {}",
                    i + 1,
                    row.get("local_address").unwrap_or(&"-".to_string()),
                    row.get("foreign_address").unwrap_or(&"-".to_string()),
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n═══════════════════════════════════\n");

    // Ejemplo 4: Resumen estadístico
    println!("4. Resumen estadístico:\n");
    match service.get_summary().await {
        Ok(summary) => {
            summary.print();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
