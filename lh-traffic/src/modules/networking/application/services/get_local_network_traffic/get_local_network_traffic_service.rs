use anyhow::Result;
use std::sync::Arc;

use crate::modules::networking::domain::types::NetworkConnection;
use crate::modules::networking::infrastructure::repositories::SystemNetworkReaderRepository;
use crate::modules::shared::infrastructure::components::logger::Logger;

/// Servicio de aplicación para obtener el tráfico de red local
/// Siguiendo el patrón del CheckAppService en TypeScript
pub struct GetLocalNetworkTrafficService {
    logger: Arc<Logger>,
    system_network_reader_repository: SystemNetworkReaderRepository,
}

impl GetLocalNetworkTrafficService {
    /// Crea una nueva instancia del servicio
    /// No es singleton, se crea cada vez que se necesita (patrón del original)
    pub fn new() -> Self {
        Self {
            logger: Logger::instance(),
            system_network_reader_repository: SystemNetworkReaderRepository::new(),
        }
    }

    /// Método principal del caso de uso
    /// Obtiene todas las conexiones de red activas
    pub async fn invoke(&self) -> Result<Vec<NetworkConnection>> {
        self.logger
            .log_info("Starting GetLocalNetworkTrafficService", "invoke")
            .await;

        let connections = self
            .system_network_reader_repository
            .get_local_network_traffic()
            .await?;

        self.logger
            .log_info(
                &format!("Retrieved {} network connections", connections.len()),
                "invoke"
            )
            .await;

        Ok(connections)
    }

    /// Obtiene solo conexiones establecidas
    pub async fn get_established_only(&self) -> Result<Vec<NetworkConnection>> {
        self.logger
            .log_debug("Getting only established connections", "get_established_only")
            .await;

        self.system_network_reader_repository
            .get_established_connections()
            .await
    }

    /// Obtiene conexiones por puerto específico
    pub async fn get_by_port(&self, port: u16) -> Result<Vec<NetworkConnection>> {
        self.logger
            .log_debug(
                &format!("Getting connections for port {}", port),
                "get_by_port"
            )
            .await;

        self.system_network_reader_repository
            .get_connections_by_local_port(port)
            .await
    }

    /// Genera un reporte resumido de las conexiones
    pub async fn get_summary(&self) -> Result<NetworkTrafficSummary> {
        let connections = self.invoke().await?;

        let tcp_count = connections
            .iter()
            .filter(|c| c.protocol.to_lowercase() == "tcp")
            .count();

        let udp_count = connections
            .iter()
            .filter(|c| c.protocol.to_lowercase() == "udp")
            .count();

        let established_count = connections
            .iter()
            .filter(|c| c.state.to_uppercase() == "ESTAB" || c.state.to_uppercase() == "ESTABLISHED")
            .count();

        let listening_count = connections
            .iter()
            .filter(|c| c.state.to_uppercase() == "LISTEN")
            .count();

        Ok(NetworkTrafficSummary {
            total_connections: connections.len(),
            tcp_connections: tcp_count,
            udp_connections: udp_count,
            established_connections: established_count,
            listening_connections: listening_count,
        })
    }
}

impl Default for GetLocalNetworkTrafficService {
    fn default() -> Self {
        Self::new()
    }
}

/// DTO con el resumen del tráfico de red
#[derive(Debug, Clone)]
pub struct NetworkTrafficSummary {
    pub total_connections: usize,
    pub tcp_connections: usize,
    pub udp_connections: usize,
    pub established_connections: usize,
    pub listening_connections: usize,
}

impl NetworkTrafficSummary {
    pub fn print(&self) {
        println!("=== Network Traffic Summary ===");
        println!("Total connections: {}", self.total_connections);
        println!("TCP connections: {}", self.tcp_connections);
        println!("UDP connections: {}", self.udp_connections);
        println!("Established: {}", self.established_connections);
        println!("Listening: {}", self.listening_connections);
        println!("==============================");
    }
}
