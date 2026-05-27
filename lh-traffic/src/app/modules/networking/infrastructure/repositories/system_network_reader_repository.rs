use std::process::Command;
use anyhow::{Result, Context};
use crate::app::modules::networking::domain::entities::NetworkConnectionEntity;
use crate::app::modules::shared::infrastructure::components::logger::Logger;
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

    /// Alias de new() para mantener compatibilidad con la API de Deno
    /// En Deno: SystemOsReaderRepository.getInstance()
    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Obtiene todas las conexiones de red activas del sistema
    /// En Linux: usa el comando `ss` (Socket Statistics)
    /// En Windows: usa el comando `netstat`
    pub async fn get_local_network_traffic(&self) -> Result<Vec<NetworkConnectionEntity>> {
        self.logger
            .log_debug("Getting local network traffic", "SystemNetworkReaderRepository")
            .await;

        // Detectar sistema operativo y usar el comando apropiado
        let (command, args, is_windows) = if cfg!(target_os = "windows") {
            // Windows: netstat -ano
            // -a: todas las conexiones
            // -n: no resolver nombres (más rápido)
            // -o: mostrar PID del proceso
            ("netstat", vec!["-ano"], true)
        } else {
            // Linux: ss -antp
            // -a: todas las conexiones
            // -n: no resolver nombres (más rápido)
            // -t: TCP
            // -p: mostrar procesos
            ("ss", vec!["-antp"], false)
        };

        let output = Command::new(command)
            .args(&args)
            .output()
            .context(format!("Failed to execute '{}' command. Make sure it's installed.", command))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command '{}' failed: {}", command, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let connections = if is_windows {
            self.parse_netstat_output(&stdout)?
        } else {
            self.parse_ss_output(&stdout)?
        };

        self.logger
            .log_debug(
                &format!("Found {} network connections", connections.len()),
                "SystemNetworkReaderRepository"
            )
            .await;

        Ok(connections)
    }

    /// Parsea la salida del comando `netstat` (Windows)
    /// Formato típico: TCP    192.168.1.100:45678    93.184.216.34:443      ESTABLISHED     1234
    fn parse_netstat_output(&self, output: &str) -> Result<Vec<NetworkConnectionEntity>> {
        let mut connections = Vec::new();

        for line in output.lines().skip(4) {
            // Saltar encabezados (primeras 4 líneas)
            if line.trim().is_empty() {
                continue;
            }

            if let Some(conn) = self.parse_netstat_line(line) {
                connections.push(conn);
            }
        }

        Ok(connections)
    }

    /// Parsea una línea individual de la salida de `netstat` (Windows)
    fn parse_netstat_line(&self, line: &str) -> Option<NetworkConnectionEntity> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 4 {
            return None;
        }

        let protocol = parts[0].to_string();
        let local_address = parts[1].to_string();
        let foreign_address = parts[2].to_string();
        let state = parts[3].to_string();

        let mut connection = NetworkConnectionEntity::new(
            protocol,
            local_address,
            foreign_address,
            state,
        );

        // Si hay PID disponible (última columna)
        if let Some(pid_str) = parts.get(4) {
            if let Ok(pid) = pid_str.parse::<u32>() {
                // En Windows netstat no muestra el nombre del proceso, solo el PID
                connection = connection.with_process_info(pid, format!("PID:{}", pid));
            }
        }

        Some(connection)
    }

    /// Parsea la salida del comando `ss`
    fn parse_ss_output(&self, output: &str) -> Result<Vec<NetworkConnectionEntity>> {
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
    fn parse_ss_line(&self, line: &str) -> Option<NetworkConnectionEntity> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 5 {
            return None;
        }

        let protocol = parts[0].to_string();
        let state = parts[1].to_string();
        let local_address = parts.get(4).unwrap_or(&"").to_string();
        let foreign_address = parts.get(5).unwrap_or(&"").to_string();

        let mut connection = NetworkConnectionEntity::new(
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
    pub async fn get_established_connections(&self) -> Result<Vec<NetworkConnectionEntity>> {
        let all_connections = self.get_local_network_traffic().await?;

        Ok(all_connections
            .into_iter()
            .filter(|conn| conn.state.to_uppercase() == "ESTAB" || conn.state.to_uppercase() == "ESTABLISHED")
            .collect())
    }

    /// Obtiene conexiones filtradas por puerto local
    pub async fn get_connections_by_local_port(&self, port: u16) -> Result<Vec<NetworkConnectionEntity>> {
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
