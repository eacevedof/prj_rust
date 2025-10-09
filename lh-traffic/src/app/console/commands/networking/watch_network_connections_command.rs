use crate::app::console::abstract_command::AbstractCommand;
use crate::modules::shared::infrastructure::components::cli::CliColor;
use crate::modules::networking::GetLocalNetworkTrafficService;
use crate::modules::networking::GetLocalNetworkTrafficInputDto;
use std::io::{self, Write};

/// Command to watch network connections with auto-refresh
/// Refreshes every 10 seconds
pub struct WatchNetworkConnectionsCommand {
    base: AbstractCommand,
}

impl WatchNetworkConnectionsCommand {
    pub fn new() -> Self {
        Self {
            base: AbstractCommand::new(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    pub async fn invoke(&mut self, filter: Option<String>, refresh_secs: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
        let refresh_interval = refresh_secs.unwrap_or(10);

        self.base.echo_start(&format!("WatchNetworkConnectionsCommand (refresh every {}s)", refresh_interval));

        CliColor::echo_yellow(&format!("Press Ctrl+C to stop watching..."));
        println!();

        let mut iteration = 0;

        loop {
            iteration += 1;

            // Clear screen (ANSI escape code)
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().unwrap();

            match self.display_connections(filter.clone(), iteration).await {
                Ok(_) => {},
                Err(e) => {
                    self.base.logger.log_error(&format!("Error: {}", e), "WatchNetworkConnectionsCommand").await;
                    CliColor::echo_red(&format!("Error: {}", e));
                }
            }

            println!();
            CliColor::echo_yellow(&format!("Next refresh in {} seconds... (Press Ctrl+C to stop)", refresh_interval));

            // Sleep for refresh interval
            self.base.sleep_seconds(refresh_interval).await;
        }
    }

    async fn display_connections(&mut self, filter: Option<String>, iteration: usize) -> Result<(), Box<dyn std::error::Error>> {
        let service = GetLocalNetworkTrafficService::get_instance();

        let input = if let Some(f) = filter {
            GetLocalNetworkTrafficInputDto::new(f)
        } else {
            GetLocalNetworkTrafficInputDto::empty()
        };

        let result = service.invoke(input).await?;

        // Header
        CliColor::echo_cyan(&format!("═══════════════════════════════════════════════════════════════════════════════"));
        CliColor::echo_cyan(&format!("  LH TRAFFIC - Network Connections Monitor (Iteration #{})  ", iteration));
        CliColor::echo_cyan(&format!("═══════════════════════════════════════════════════════════════════════════════"));

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        CliColor::echo_green(&format!("Last update: {} | Total connections: {}", now, result.total));

        println!();

        // Table header
        CliColor::echo_cyan(&format!(
            "{:<10} {:<25} {:<25} {:<15} {:<10} {:<15}",
            "PROTOCOL", "LOCAL ADDRESS", "FOREIGN ADDRESS", "STATE", "PID", "PROGRAM"
        ));
        CliColor::echo_cyan("───────────────────────────────────────────────────────────────────────────────");

        // Connections
        for row in &result.rows {
            let protocol = row.get("protocol").map(|s| s.as_str()).unwrap_or("-");
            let local_addr = row.get("local_address").map(|s| s.as_str()).unwrap_or("-");
            let foreign_addr = row.get("foreign_address").map(|s| s.as_str()).unwrap_or("-");
            let state = row.get("state").map(|s| s.as_str()).unwrap_or("-");
            let pid = row.get("pid").map(|s| s.as_str()).unwrap_or("-");
            let program = row.get("program_name").map(|s| s.as_str()).unwrap_or("-");

            let line = format!(
                "{:<10} {:<25} {:<25} {:<15} {:<10} {:<15}",
                protocol,
                Self::truncate(local_addr, 25),
                Self::truncate(foreign_addr, 25),
                state,
                pid,
                Self::truncate(program, 15)
            );

            // Color by state
            if state.to_uppercase().contains("ESTAB") {
                CliColor::echo_green(&line);
            } else if state.to_uppercase().contains("LISTEN") {
                CliColor::echo_yellow(&line);
            } else {
                println!("{}", line);
            }
        }

        println!();
        CliColor::echo_cyan("═══════════════════════════════════════════════════════════════════════════════");

        Ok(())
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() > max_len {
            format!("{}...", &s[..max_len - 3])
        } else {
            s.to_string()
        }
    }
}
