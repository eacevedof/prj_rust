use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// DTO de salida para el caso de uso GetLocalNetworkTraffic
/// Representa las filas como un array de HashMaps (como array asociativo en PHP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotLocalNetworkTrafficDto {
    /// Lista de filas, cada fila es un HashMap con los campos
    /// Equivalente a: array<int, array<string, string>> en PHP
    pub rows: Vec<HashMap<String, String>>,

    /// Número total de filas
    pub total: usize,
}

impl GotLocalNetworkTrafficDto {
    pub fn new(rows: Vec<HashMap<String, String>>) -> Self {
        let total = rows.len();
        Self { rows, total }
    }

    pub fn empty() -> Self {
        Self {
            rows: Vec::new(),
            total: 0,
        }
    }

    /// Convierte a JSON para facilitar la serialización
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// Tipo alias para las filas (similar a PHP)
// PHP: array<string, string>
pub type Row = HashMap<String, String>;
