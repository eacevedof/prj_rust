use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::time::Duration;


/// Información de geolocalización de una IP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpGeolocationInfo {
    /// IP consultada
    pub query: String,

    /// País
    pub country: String,

    /// Código del país (ej: "ES", "US")
    pub country_code: String,

    /// Región/Provincia
    pub region: String,

    /// Ciudad
    pub city: String,

    /// ISP
    pub isp: String,

    /// Organización
    pub org: String,

    /// Latitud
    pub lat: f64,

    /// Longitud
    pub lon: f64,
}

/// Response de la API ip-api.com
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IpApiResponse {
    query: String,
    status: String,
    country: Option<String>,
    country_code: Option<String>,
    _region: Option<String>,
    region_name: Option<String>,
    city: Option<String>,
    isp: Option<String>,
    org: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    _message: Option<String>,
}

/// Repositorio para obtener información de geolocalización de IPs
/// Usa el servicio gratuito de ip-api.com
///
/// Limitaciones del servicio gratuito:
/// - 45 requests por minuto
/// - Solo HTTP (no HTTPS)
///
/// Para uso en producción considera:
/// - Implementar caché local con Redis
/// - Usar versión Pro con HTTPS
/// - O usar otro servicio como MaxMind GeoIP2
pub struct IpGeolocationRepository {
    client: reqwest::Client,
}

impl IpGeolocationRepository {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Obtiene información de geolocalización para una IP
    ///
    /// # Arguments
    /// * `ip` - Dirección IP (ej: "8.8.8.8")
    ///
    /// # Returns
    /// * `Ok(Some(IpGeolocationInfo))` - Si se encontró información
    /// * `Ok(None)` - Si la IP es local o privada
    /// * `Err` - Si hubo un error en la consulta
    pub async fn get_geolocation(&self, ip: &str) -> Result<Option<IpGeolocationInfo>> {
        // Filtrar IPs locales/privadas (sin log)
        if self.is_local_or_private_ip(ip) {
            return Ok(None);
        }

        let url = format!("http://ip-api.com/json/{}", ip);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to ip-api.com")?;

        if !response.status().is_success() {
            anyhow::bail!("ip-api.com returned error status: {}", response.status());
        }

        let api_response: IpApiResponse = response
            .json()
            .await
            .context("Failed to parse ip-api.com response")?;

        if api_response.status != "success" {
            return Ok(None);
        }

        let info = IpGeolocationInfo {
            query: api_response.query,
            country: api_response.country.unwrap_or_else(|| "Unknown".to_string()),
            country_code: api_response.country_code.unwrap_or_else(|| "??".to_string()),
            region: api_response.region_name.unwrap_or_else(|| "Unknown".to_string()),
            city: api_response.city.unwrap_or_else(|| "Unknown".to_string()),
            isp: api_response.isp.unwrap_or_else(|| "Unknown".to_string()),
            org: api_response.org.unwrap_or_else(|| "Unknown".to_string()),
            lat: api_response.lat.unwrap_or(0.0),
            lon: api_response.lon.unwrap_or(0.0),
        };

        Ok(Some(info))
    }

    /// Extrae solo la IP de un string "IP:Puerto"
    /// Ej: "93.184.216.34:443" -> "93.184.216.34"
    pub fn extract_ip_from_address(&self, address: &str) -> String {
        address
            .split(':')
            .next()
            .unwrap_or(address)
            .to_string()
    }

    /// Verifica si una IP es local o privada
    fn is_local_or_private_ip(&self, ip: &str) -> bool {
        // Loopback
        if ip.starts_with("127.") || ip == "::1" || ip.starts_with("::ffff:127.") {
            return true;
        }

        // IPv4 privadas (RFC 1918)
        if ip.starts_with("10.")
            || ip.starts_with("192.168.")
            || ip.starts_with("172.16.")
            || ip.starts_with("172.17.")
            || ip.starts_with("172.18.")
            || ip.starts_with("172.19.")
            || ip.starts_with("172.20.")
            || ip.starts_with("172.21.")
            || ip.starts_with("172.22.")
            || ip.starts_with("172.23.")
            || ip.starts_with("172.24.")
            || ip.starts_with("172.25.")
            || ip.starts_with("172.26.")
            || ip.starts_with("172.27.")
            || ip.starts_with("172.28.")
            || ip.starts_with("172.29.")
            || ip.starts_with("172.30.")
            || ip.starts_with("172.31.") {
            return true;
        }

        // 0.0.0.0 (wildcard)
        if ip == "0.0.0.0" || ip == "::" || ip == "*" {
            return true;
        }

        false
    }

    /// Obtiene solo el código de país (más rápido, menos información)
    pub async fn get_country_code(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_geolocation(ip).await? {
            Ok(Some(info.country_code))
        } else {
            Ok(None)
        }
    }

    /// Obtiene el país completo (ej: "Spain", "United States")
    pub async fn get_country(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_geolocation(ip).await? {
            Ok(Some(info.country))
        } else {
            Ok(None)
        }
    }
}

impl Default for IpGeolocationRepository {
    fn default() -> Self {
        Self::new()
    }
}
