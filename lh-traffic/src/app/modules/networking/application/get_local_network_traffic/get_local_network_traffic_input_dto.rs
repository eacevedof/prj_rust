use serde::{Serialize, Deserialize};

/// DTO de entrada para el caso de uso GetLocalNetworkTraffic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLocalNetworkTrafficInputDto {
    /// Filtro para buscar en las conexiones (opcional)
    /// Filtra por: dirección local, dirección remota, nombre del programa, etc.
    pub filter: String,
}

impl GetLocalNetworkTrafficInputDto {
    pub fn new(filter: String) -> Self {
        Self { filter }
    }

    pub fn empty() -> Self {
        Self {
            filter: String::new(),
        }
    }
}
