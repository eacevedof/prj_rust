use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use crate::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    HybridIpInfoRepository,
    ProcessInfoRepository,
};
use crate::app::modules::networking::domain::entities::NetworkConnectionEntity;
use crate::app::console::abstract_command::AbstractCommand;

/// Command to list all network connections with country and organization info
pub struct ListNetworkConnectionsCommand {
    base: AbstractCommand,
}

impl ListNetworkConnectionsCommand {
    pub fn new() -> Self {
        Self {
            base: AbstractCommand::new(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    pub async fn invoke(&mut self, filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.base.echo_start("ListNetworkConnectionsCommand");

        match self.execute(filter).await {
            Ok(_) => {},
            Err(e) => {
                self.base.logger.log_error(&format!("Error: {}", e), "ListNetworkConnectionsCommand").await;
                CliColor::die_red(&format!("Error: {}", e));
            }
        }

        self.base.echo_end("ListNetworkConnectionsCommand");
        Ok(())
    }

    async fn execute(&mut self, filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let system_network_reader_repository: SystemNetworkReaderRepository = SystemNetworkReaderRepository::get_instance();
        let hybrid_ip_info_repository: HybridIpInfoRepository = HybridIpInfoRepository::get_instance();
        let process_info_repository: ProcessInfoRepository = ProcessInfoRepository::get_instance();

        // Get connections
        let mut connections: Vec<NetworkConnectionEntity> = system_network_reader_repository.get_local_network_traffic().await?;

        // Apply filter if provided (supports multiple filters with comma: "established,remote")
        if let Some(ref filter_text) = filter {
            let filters: Vec<String> = filter_text
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect();

            connections.retain(|conn| {
                // All filters must match (AND logic)
                filters.iter().all(|filter_lower| {
                    // Special filter: "remote" = exclude local IPs
                    if filter_lower == "remote" {
                        let remote_ip: String = hybrid_ip_info_repository.extract_ip_from_address(&conn.foreign_address);
                        return !remote_ip.starts_with("127.")
                            && !remote_ip.starts_with("192.168.")
                            && !remote_ip.starts_with("0.0.0.0")
                            && remote_ip != "*";
                    }

                    // General filter: search in all fields (OR logic within the filter)
                    conn.protocol.to_lowercase().contains(filter_lower)
                        || conn.local_address.to_lowercase().contains(filter_lower)
                        || conn.foreign_address.to_lowercase().contains(filter_lower)
                        || conn.state.to_lowercase().contains(filter_lower)
                        || conn.program_name.as_ref().map_or(false, |p| p.to_lowercase().contains(filter_lower))
                })
            });
        }

        self.base.echo_step(&format!("Total connections found: {}", connections.len()));

        if connections.is_empty() {
            CliColor::echo_yellow("No connections found.");
            return Ok(());
        }

        // Print header
        println!();
        CliColor::echo_cyan("=".repeat(150).as_str());
        CliColor::echo_cyan(&format!(
            "{:<8} {:<22} {:<22} {:<12} {:<28} {:<12} {:<30}",
            "PROTOCOL", "LOCAL ADDRESS", "FOREIGN ADDRESS", "STATE", "PID - PROGRAM", "COUNTRY", "ORGANIZATION"
        ));
        CliColor::echo_cyan("=".repeat(150).as_str());

        // Process each connection
        for (idx, conn) in connections.iter().enumerate() {
            let protocol = &conn.protocol;
            let local_addr = &conn.local_address;
            let foreign_addr = &conn.foreign_address;
            let state = &conn.state;

            // Get process name (format: PID:name)
            let program: String = if let Some(p) = conn.pid {
                // Always try to get real process name from ProcessInfoRepository
                let name: String = if let Ok(Some(info)) = process_info_repository.get_process_name(p).await {
                    info
                } else if let Some(ref n) = conn.program_name {
                    // Fallback to program_name from netstat/ss (if available)
                    n.clone()
                } else {
                    "?".to_string()
                };
                format!("{} - {}", p, name)
            } else {
                "-".to_string()
            };

            // Get country and organization from foreign IP (only for remote IPs)
            let remote_ip: String = hybrid_ip_info_repository.extract_ip_from_address(foreign_addr);
            let is_remote: bool = !remote_ip.starts_with("127.")
                && !remote_ip.starts_with("0.0.0.0")
                && remote_ip != "0.0.0.0"
                && remote_ip != "*";

            let (country, organization): (String, String) = if is_remote {
                if let Ok(Some(hybrid_ip_info)) = hybrid_ip_info_repository.get_ip_info(&remote_ip).await {
                    (
                        hybrid_ip_info.country_code.unwrap_or(hybrid_ip_info.country),
                        hybrid_ip_info.organization
                    )
                } else {
                    ("-".to_string(), "-".to_string())
                }
            } else {
                ("-".to_string(), "-".to_string())
            };

            // Print row
            let line = format!(
                "{:<8} {:<22} {:<22} {:<12} {:<28} {:<12} {:<30}",
                Self::truncate(protocol, 8),
                Self::truncate(local_addr, 22),
                Self::truncate(foreign_addr, 22),
                Self::truncate(state, 12),
                Self::truncate(&program, 28),
                Self::truncate(&country, 12),
                Self::truncate(&organization, 30)
            );

            // Color by state
            if state.to_uppercase().contains("ESTAB") {
                CliColor::echo_green(&line);
            } else if state.to_uppercase().contains("LISTEN") {
                CliColor::echo_yellow(&line);
            } else {
                println!("{}", line);
            }

            // Rate limiting: only for remote IPs
            if is_remote && idx < connections.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        }

        println!();
        CliColor::echo_cyan("=".repeat(150).as_str());
        self.base.echo_step(&format!("Total displayed: {}", connections.len()));

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
