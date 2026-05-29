use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use crate::app::modules::networking::domain::entities::NetworkConnectionEntity;
use crate::app::modules::networking::infrastructure::repositories::SystemNetworkReaderRepository;
use super::{
    GetLocalNetworkTrafficInputDto,
    GotLocalNetworkTrafficDto,
    Row,
    NetworkTrafficSummaryDto,
};
use crate::app::modules::shared::infrastructure::components::logger::Logger;

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
            system_network_reader_repository: SystemNetworkReaderRepository::get_instance(),
        }
    }

    /// Alias de new() para mantener compatibilidad con la API de Deno
    /// En Deno: CheckAppService.getInstance()
    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Método principal del caso de uso
    /// Recibe un DTO de entrada con filtro y devuelve un DTO con filas
    pub async fn invoke(&self, get_local_network_traffic_input_dto: GetLocalNetworkTrafficInputDto) -> Result<GotLocalNetworkTrafficDto> {
        self.logger
            .log_info(
                &format!("Starting GetLocalNetworkTrafficService with filter: '{}'", get_local_network_traffic_input_dto.filter),
                "invoke"
            )
            .await;

        // Obtener todas las conexiones del sistema
        let network_connection_entities = self
            .system_network_reader_repository
            .get_local_network_traffic()
            .await?;

        // Aplicar filtro si existe
        let filtered_network_connection_entities = if get_local_network_traffic_input_dto.filter.is_empty() {
            network_connection_entities
        } else {
            self.apply_filter(network_connection_entities, &get_local_network_traffic_input_dto.filter)
        };

        // Convertir las conexiones a filas (HashMap)
        let rows = self.connections_to_rows(filtered_network_connection_entities);

        self.logger
            .log_info(
                &format!("Retrieved {} network connections", rows.len()),
                "invoke"
            )
            .await;

        Ok(GotLocalNetworkTrafficDto::new(rows))
    }

    /// Aplica el filtro a las conexiones
    /// Busca el filtro en todos los campos de la conexión
    fn apply_filter(&self, network_connection_entities: Vec<NetworkConnectionEntity>, filter: &str) -> Vec<NetworkConnectionEntity> {
        let filter_lower: String = filter.to_lowercase();

        network_connection_entities
            .into_iter()
            .filter(|network_connection_entity| {
                // Buscar en todos los campos
                network_connection_entity.protocol.to_lowercase().contains(&filter_lower)
                    || network_connection_entity.local_address.to_lowercase().contains(&filter_lower)
                    || network_connection_entity.foreign_address.to_lowercase().contains(&filter_lower)
                    || network_connection_entity.state.to_lowercase().contains(&filter_lower)
                    || network_connection_entity.program_name
                        .as_ref()
                        .map(|p| p.to_lowercase().contains(&filter_lower))
                        .unwrap_or(false)
                    || network_connection_entity.pid
                        .map(|p| p.to_string().contains(filter))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Convierte NetworkConnectionEntity a Row (HashMap<String, String>)
    /// Similar a convertir a array asociativo en PHP
    fn connections_to_rows(&self, network_connection_entities: Vec<NetworkConnectionEntity>) -> Vec<Row> {
        network_connection_entities
            .into_iter()
            .map(|network_connection_entity| {
                let mut row: Row = HashMap::new();

                row.insert("protocol".to_string(), network_connection_entity.protocol);
                row.insert("local_address".to_string(), network_connection_entity.local_address);
                row.insert("foreign_address".to_string(), network_connection_entity.foreign_address);
                row.insert("state".to_string(), network_connection_entity.state);
                row.insert(
                    "pid".to_string(),
                    network_connection_entity.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()),
                );
                row.insert(
                    "program_name".to_string(),
                    network_connection_entity.program_name.unwrap_or_else(|| "-".to_string()),
                );

                row
            })
            .collect()
    }

    /// Obtiene solo conexiones establecidas
    pub async fn get_established_only(&self) -> Result<Vec<NetworkConnectionEntity>> {
        self.logger
            .log_debug("Getting only established connections", "get_established_only")
            .await;

        self.system_network_reader_repository
            .get_established_connections()
            .await
    }

    /// Obtiene conexiones por puerto específico
    pub async fn get_by_port(&self, port: u16) -> Result<Vec<NetworkConnectionEntity>> {
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
    pub async fn get_summary(&self) -> Result<NetworkTrafficSummaryDto> {
        let get_local_network_traffic_input_dto = GetLocalNetworkTrafficInputDto::empty();
        let got_local_network_traffic_dto = self.invoke(get_local_network_traffic_input_dto).await?;

        let tcp_count = got_local_network_traffic_dto
            .rows
            .iter()
            .filter(|row| {
                row.get("protocol")
                    .map(|p| p.to_lowercase() == "tcp")
                    .unwrap_or(false)
            })
            .count();

        let udp_count = got_local_network_traffic_dto
            .rows
            .iter()
            .filter(|row| {
                row.get("protocol")
                    .map(|p| p.to_lowercase() == "udp")
                    .unwrap_or(false)
            })
            .count();

        let established_count = got_local_network_traffic_dto
            .rows
            .iter()
            .filter(|row| {
                row.get("state")
                    .map(|s| {
                        let state = s.to_uppercase();
                        state == "ESTAB" || state == "ESTABLISHED"
                    })
                    .unwrap_or(false)
            })
            .count();

        let listening_count = got_local_network_traffic_dto
            .rows
            .iter()
            .filter(|row| {
                row.get("state")
                    .map(|s| s.to_uppercase() == "LISTEN")
                    .unwrap_or(false)
            })
            .count();

        Ok(NetworkTrafficSummaryDto {
            total_connections: got_local_network_traffic_dto.total,
            tcp_connections: tcp_count,
            udp_connections: udp_count,
            established_connections: established_count,
            listening_connections: listening_count,
        })
    }
}
