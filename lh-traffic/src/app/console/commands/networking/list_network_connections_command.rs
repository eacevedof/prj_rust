use crate::app::console::abstract_command::AbstractCommand;
use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use crate::app::modules::networking::GetLocalNetworkTrafficService;
use crate::app::modules::networking::GetLocalNetworkTrafficInputDto;

/// Command to list all network connections
/// Equivalent to: ETLRefreshDomainsInRedisCommand pattern
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
        let service = GetLocalNetworkTrafficService::get_instance();

        let input = if let Some(f) = filter {
            GetLocalNetworkTrafficInputDto::new(f)
        } else {
            GetLocalNetworkTrafficInputDto::empty()
        };

        let result = service.invoke(input).await?;

        self.base.echo_step(&format!("Total connections found: {}", result.total));

        // Imprimir cabecera
        println!();
        CliColor::echo_cyan("================================================================================");
        CliColor::echo_cyan(&format!(
            "{:<10} {:<25} {:<25} {:<15} {:<10} {:<15}",
            "PROTOCOL", "LOCAL ADDRESS", "FOREIGN ADDRESS", "STATE", "PID", "PROGRAM"
        ));
        CliColor::echo_cyan("================================================================================");

        // Imprimir cada conexión
        for row in &result.rows {
            let protocol = row.get("protocol").map(|s| s.as_str()).unwrap_or("-");
            let local_addr = row.get("local_address").map(|s| s.as_str()).unwrap_or("-");
            let foreign_addr = row.get("foreign_address").map(|s| s.as_str()).unwrap_or("-");
            let state = row.get("state").map(|s| s.as_str()).unwrap_or("-");
            let pid = row.get("pid").map(|s| s.as_str()).unwrap_or("-");
            let program = row.get("program_name").map(|s| s.as_str()).unwrap_or("-");

            // Colorear según el estado
            let line = format!(
                "{:<10} {:<25} {:<25} {:<15} {:<10} {:<15}",
                protocol,
                Self::truncate(local_addr, 25),
                Self::truncate(foreign_addr, 25),
                state,
                pid,
                Self::truncate(program, 15)
            );

            if state.to_uppercase().contains("ESTAB") {
                CliColor::echo_green(&line);
            } else if state.to_uppercase().contains("LISTEN") {
                CliColor::echo_yellow(&line);
            } else {
                println!("{}", line);
            }
        }

        println!();
        CliColor::echo_cyan("================================================================================");
        self.base.echo_step(&format!("Total displayed: {}", result.total));

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
