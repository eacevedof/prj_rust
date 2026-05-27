use lh_traffic::app::console::commands::networking::ListEnhancedNetworkConnectionsCommand;
use std::env;

#[tokio::main]
async fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // First argument after program name is the filter (optional)
    let filter = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    let mut command = ListEnhancedNetworkConnectionsCommand::get_instance();

    if let Err(e) = command.invoke(filter).await {
        eprintln!("Command failed: {}", e);
        std::process::exit(1);
    }
}
