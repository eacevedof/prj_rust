use std::env;
use lh_traffic::app;

//tokio permite ejecución asincrona
#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Check if running as console command
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // Run console command
        app::console::run_console(args).await;
        return;
    }

    // Show help if no command provided
    println!("LH Traffic - Network Traffic Monitor");
    println!();
    println!("Usage:");
    println!("  lh-traffic <command> [arguments]");
    println!();
    println!("Available commands:");
    println!("  list-connections [filter]  - List network connections");
    println!("  watch-connections [filter] - Watch connections (auto-refresh every 10s)");
    println!();
    println!("Examples:");
    println!("  lh-traffic list-connections");
    println!("  lh-traffic list-connections firefox");
    println!("  lh-traffic watch-connections");
    println!();
    println!("Or use dedicated binaries:");
    println!("  cargo run --bin list-network-connections");
    println!("  cargo run --bin watch-network-connections");
}
