/// Binario de ejemplo para probar el módulo Networking
/// Ejecutar con: cargo run --bin network_traffic

use lh_traffic::modules::networking::GetLocalNetworkTrafficService;

#[tokio::main]
async fn main() {
    // Inicializar tracing
    tracing_subscriber::fmt::init();

    println!("=== LH Traffic - Network Monitor ===\n");

    // Crear el servicio
    let service = GetLocalNetworkTrafficService::new();

    // Obtener todas las conexiones
    match service.invoke().await {
        Ok(connections) => {
            println!("Found {} active connections:\n", connections.len());

            for (i, conn) in connections.iter().enumerate().take(20) {
                // Mostrar solo las primeras 20
                println!(
                    "{}. {} | {} -> {} | State: {} | PID: {:?} | Program: {:?}",
                    i + 1,
                    conn.protocol,
                    conn.local_address,
                    conn.foreign_address,
                    conn.state,
                    conn.pid,
                    conn.program_name
                );
            }

            if connections.len() > 20 {
                println!("\n... and {} more connections", connections.len() - 20);
            }
        }
        Err(e) => {
            eprintln!("Error getting network traffic: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n");

    // Obtener resumen
    match service.get_summary().await {
        Ok(summary) => {
            summary.print();
        }
        Err(e) => {
            eprintln!("Error getting summary: {}", e);
        }
    }

    println!("\n");

    // Obtener solo conexiones establecidas
    match service.get_established_only().await {
        Ok(connections) => {
            println!("Established connections: {}", connections.len());
            for conn in connections.iter().take(10) {
                println!(
                    "  {} -> {} | PID: {:?} | Program: {:?}",
                    conn.local_address, conn.foreign_address, conn.pid, conn.program_name
                );
            }
        }
        Err(e) => {
            eprintln!("Error getting established connections: {}", e);
        }
    }
}
