mod app;
mod modules;

use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Check if running as console command
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Run console command
        app::console::run_console(args).await;
    } else {
        // Run web server
        app::http_server::run_server().await;
    }
}
