use std::process::Command;
use anyhow::{Result, Context};
use crate::modules::networking::domain::types::NetworkConnection;
use crate::modules::shared::infrastructure::components::logger::Logger;
use std::sync::Arc;

/// Repositorio para leer información de red del sistema
/// Similar a SystemOsReaderRepository en TypeScript
pub struct SystemNetworkReaderRepository {
    logger: Arc<Logger>,
}

impl SystemNetworkReaderRepository {
    /// Crea una nueva instancia (no es singleton, siguiendo el patrón del original)
    pub fn new() -> Self {
        Self {
            logger: Logger::instance(),
        }
    }

    /// Obtiene todas las conexiones de red activas del sistema
    /// Usa el comando `ss` (Socket Statistics) en Linux
    /// Equivalente a `netstat` pero más moderno
    pub async fn get_local_network_traffic(&self) -> Result<Vec<NetworkConnection>> {
        self.logger
            .log_debug("Getting local network traffic", "SystemNetworkReaderRepository")
            .await;

        // Comando ss: muestra todas las conexiones TCP/UDP con información de procesos
        // -a: todas las conexiones
        // -n: no resolver nombres (más rápido)
        // -t: TCP
        // -u: UDP
        // -p: mostrar procesos
        let output = Command::new("ss")
            .args(&["-antp"])
            .output()
            .context("Failed to execute 'ss' command. Make sure it's installed.")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command 'ss' failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let connections = self.parse_ss_output(&stdout)?;

        self.logger
            .log_debug(
                &format!("Found {} network connections", connections.len()),
                "SystemNetworkReaderRepository"
            )
            .await;

        Ok(connections)
    }

    /// Parsea la salida del comando `ss`
    fn parse_ss_output(&self, output: &str) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();

        for line in output.lines().skip(1) {
            // Saltar la línea de encabezado
            if line.trim().is_empty() {
                continue;
            }

            if let Some(conn) = self.parse_ss_line(line) {
                connections.push(conn);
            }
        }

        Ok(connections)
    }

    /// Parsea una línea individual de la salida de `ss`
    /// Formato típico: tcp   ESTAB  0  0  192.168.1.100:45678  93.184.216.34:443  users:(("firefox",pid=1234,fd=56))
    fn parse_ss_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 5 {
            return None;
        }

        let protocol = parts[0].to_string();
        let state = parts[1].to_string();
        let local_address = parts.get(4).unwrap_or(&"").to_string();
        let foreign_address = parts.get(5).unwrap_or(&"").to_string();

        let mut connection = NetworkConnection::new(
            protocol,
            local_address,
            foreign_address,
            state,
        );

        // Intentar extraer información del proceso
        // Buscar la parte users:(("programa",pid=1234,fd=56))
        if let Some(process_info) = line.split("users:((").nth(1) {
            if let Some((program, rest)) = process_info.split_once("\",pid=") {
                let program_name = program.trim_start_matches('"').to_string();
                if let Some(pid_str) = rest.split(',').next() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        connection = connection.with_process_info(pid, program_name);
                    }
                }
            }
        }

        Some(connection)
    }

    /// Obtiene solo las conexiones establecidas (ESTABLISHED)
    pub async fn get_established_connections(&self) -> Result<Vec<NetworkConnection>> {
        let all_connections = self.get_local_network_traffic().await?;

        Ok(all_connections
            .into_iter()
            .filter(|conn| conn.state.to_uppercase() == "ESTAB" || conn.state.to_uppercase() == "ESTABLISHED")
            .collect())
    }

    /// Obtiene conexiones filtradas por puerto local
    pub async fn get_connections_by_local_port(&self, port: u16) -> Result<Vec<NetworkConnection>> {
        let all_connections = self.get_local_network_traffic().await?;

        Ok(all_connections
            .into_iter()
            .filter(|conn| {
                conn.local_address
                    .split(':')
                    .last()
                    .and_then(|p| p.parse::<u16>().ok())
                    == Some(port)
            })
            .collect())
    }
}

impl Default for SystemNetworkReaderRepository {
    fn default() -> Self {
        Self::new()
    }
}
