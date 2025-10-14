use lh_traffic::app::console::commands::stresser::StressAntiPhishingDomainCommand;
use std::env;

#[tokio::main]
async fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    // Usage: stress_anti_phishing_domain [requests_per_second] [duration_seconds] [api_url] [device_auth_token]
    let requests_per_second = if args.len() > 1 {
        args[1].parse::<u64>().unwrap_or(10)
    } else {
        10 // Default: 10 requests per second
    };

    let duration_seconds = if args.len() > 2 {
        args[2].parse::<u64>().unwrap_or(10)
    } else {
        10 // Default: 10 seconds
    };

    let api_url = if args.len() > 3 {
        Some(args[3].clone())
    } else {
        None // Will use default
    };

    let device_auth_token = if args.len() > 4 {
        Some(args[4].clone())
    } else {
        None // Will use default
    };

    let custom_domains: Vec<String> = Vec::new(); // For now, use default domains

    let mut command = StressAntiPhishingDomainCommand::get_instance();

    if let Err(e) = command
        .invoke(
            api_url,
            device_auth_token,
            requests_per_second,
            duration_seconds,
            custom_domains,
        )
        .await
    {
        eprintln!("Command failed: {}", e);
        std::process::exit(1);
    }
}
