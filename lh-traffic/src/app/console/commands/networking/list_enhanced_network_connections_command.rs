use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use crate::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    HybridIpInfoRepository,
    ProcessInfoRepository,
};
use crate::app::modules::networking::domain::entities::NetworkConnectionEntity;
use crate::app::console::abstract_command::AbstractCommand;

/// Command to list network connections with enhanced info (country, organization, full path)
pub struct ListEnhancedNetworkConnectionsCommand {
    base: AbstractCommand,
}

impl ListEnhancedNetworkConnectionsCommand {
    pub fn new() -> Self {
        Self {
            base: AbstractCommand::new(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    pub async fn invoke(&mut self, filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.base.echo_start("ListEnhancedNetworkConnectionsCommand");

        match self.execute(filter).await {
            Ok(_) => {},
            Err(e) => {
                self.base.logger.log_error(&format!("Error: {}", e), "ListEnhancedNetworkConnectionsCommand").await;
                CliColor::die_red(&format!("Error: {}", e));
            }
        }

        self.base.echo_end("ListEnhancedNetworkConnectionsCommand");
        Ok(())
    }

    async fn execute(&mut self, filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let system_network_reader_repository: SystemNetworkReaderRepository = SystemNetworkReaderRepository::get_instance();
        let hybrid_ip_info_repository: HybridIpInfoRepository = HybridIpInfoRepository::get_instance();
        let process_info_repository: ProcessInfoRepository = ProcessInfoRepository::get_instance();

        // Get connections
        let mut connections: Vec<NetworkConnectionEntity> = system_network_reader_repository.get_local_network_traffic().await?;

        // Apply filter if provided
        if let Some(ref filter_text) = filter {
            let filter_lower: String = filter_text.to_lowercase();
            connections.retain(|conn| {
                conn.protocol.to_lowercase().contains(&filter_lower)
                    || conn.local_address.to_lowercase().contains(&filter_lower)
                    || conn.foreign_address.to_lowercase().contains(&filter_lower)
                    || conn.state.to_lowercase().contains(&filter_lower)
                    || conn.program_name.as_ref().map_or(false, |p| p.to_lowercase().contains(&filter_lower))
            });
        }

        self.base.echo_step(&format!("Total connections found: {}", connections.len()));

        if connections.is_empty() {
            CliColor::echo_yellow("No connections found.");
            return Ok(());
        }

        // Print header
        println!();
        CliColor::echo_cyan("=".repeat(140).as_str());
        CliColor::echo_cyan(&format!(
            "{:<8} {:<22} {:<22} {:<12} {:<6} {:<20} {:<12} {:<25}",
            "PROTOCOL", "LOCAL ADDRESS", "FOREIGN ADDRESS", "STATE", "PID", "PROGRAM", "COUNTRY", "ORGANIZATION"
        ));
        CliColor::echo_cyan("=".repeat(140).as_str());

        // Process each connection
        for (idx, conn) in connections.iter().enumerate() {
            let protocol: &String = &conn.protocol;
            let local_addr: &String = &conn.local_address;
            let foreign_addr: &String = &conn.foreign_address;
            let state: &String = &conn.state;
            let pid: String = conn.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());

            // Get process name (either from connection or query by PID)
            let program: String = if let Some(ref name) = conn.program_name {
                name.clone()
            } else if let Some(p) = conn.pid {
                if let Ok(Some(info)) = process_info_repository.get_process_name(p).await {
                    info
                } else {
                    "-".to_string()
                }
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
                "{:<8} {:<22} {:<22} {:<12} {:<6} {:<20} {:<12} {:<25}",
                Self::truncate(protocol, 8),
                Self::truncate(local_addr, 22),
                Self::truncate(foreign_addr, 22),
                Self::truncate(state, 12),
                Self::truncate(&pid, 6),
                Self::truncate(&program, 20),
                Self::truncate(&country, 12),
                Self::truncate(&organization, 25)
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
        CliColor::echo_cyan("=".repeat(140).as_str());
        self.base.echo_step(&format!("Total displayed: {}", connections.len()));
        println!();
        println!("INFO: Enhanced view uses hybrid whois + ip-api.com for geolocation");
        println!("      Whois is used first (no rate limit), ip-api only when needed");

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
