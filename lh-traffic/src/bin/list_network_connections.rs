use lh_traffic::app::console::commands::networking::ListNetworkConnectionsCommand;
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

    let mut command = ListNetworkConnectionsCommand::get_instance();

    if let Err(e) = command.invoke(filter).await {
        eprintln!("Command failed: {}", e);
        std::process::exit(1);
    }
}
