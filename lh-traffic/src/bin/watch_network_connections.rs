use lh_traffic::app::console::commands::networking::WatchNetworkConnectionsCommand;
use std::env;

#[tokio::main]
async fn main() {
    // Obtener argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();

    // El primer argumento después del nombre del programa es el filtro (opcional)
    let filter = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    // El segundo argumento puede ser el intervalo de refresh (opcional)
    let refresh_secs = if args.len() > 2 {
        args[2].parse::<u64>().ok()
    } else {
        None
    };

    let mut command = WatchNetworkConnectionsCommand::get_instance();

    if let Err(e) = command.invoke(filter, refresh_secs).await {
        eprintln!("Command failed: {}", e);
        std::process::exit(1);
    }
}
