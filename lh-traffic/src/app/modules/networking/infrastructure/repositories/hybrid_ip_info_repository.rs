use anyhow::Result;
use serde::{Serialize, Deserialize};
use super::{WhoisRepository, WhoisInfo, IpGeolocationRepository, IpGeolocationInfo, IpCacheRepository};

/// Información combinada de IP (whois + geolocalización)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridIpInfo {
    /// IP consultada
    pub ip: String,

    /// País (prioridad: ip-api > whois)
    pub country: String,

    /// Código del país (solo de ip-api)
    pub country_code: Option<String>,

    /// Ciudad (solo de ip-api)
    pub city: Option<String>,

    /// Organización propietaria (prioridad: whois > ip-api.isp)
    pub organization: String,

    /// ISP (solo de ip-api)
    pub isp: Option<String>,

    /// ASN (solo de whois)
    pub asn: Option<String>,

    /// Rango de IPs (solo de whois)
    pub ip_range: Option<String>,

    /// Método usado: "whois", "api", "hybrid", "cache"
    pub source: String,
}

/// Repositorio híbrido que combina whois + ip-api.com + caché JSON
///
/// Estrategia:
/// 1. Verificar caché primero (15 días de TTL)
/// 2. Si no está en caché o expiró:
///    a. Consulta whois primero (rápido, sin límite, da organización)
///    b. Si whois NO devuelve país, consulta ip-api.com
///    c. Combina la mejor información de ambos
///    d. Guarda en caché
///
/// Ventajas:
/// - Caché de 15 días evita consultas repetidas
/// - Sin límite de rate (usa api solo cuando es necesario)
/// - Información completa (organización + geolocalización)
/// - Optimizado para velocidad
pub struct HybridIpInfoRepository {
    whois_repo: WhoisRepository,
    geo_repo: IpGeolocationRepository,
    cache_repo: IpCacheRepository,
}

impl HybridIpInfoRepository {
    pub fn new() -> Self {
        Self {
            whois_repo: WhoisRepository::new(),
            geo_repo: IpGeolocationRepository::new(),
            cache_repo: IpCacheRepository::new(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Obtiene información combinada de una IP
    ///
    /// # Strategy
    /// 1. Check cache first (15 days TTL)
    /// 2. If not in cache or expired:
    ///    a. Query whois first (no rate limit, fast)
    ///    b. If whois has country -> use it, done
    ///    c. If whois has NO country -> query ip-api for geolocation
    ///    d. Combine best info from both sources
    ///    e. Save to cache
    ///
    /// # Arguments
    /// * `ip` - IP address (e.g., "8.8.8.8")
    ///
    /// # Returns
    /// * `Ok(Some(HybridIpInfo))` - Combined info found
    /// * `Ok(None)` - Local/private IP
    /// * `Err` - Query failed
    pub async fn get_ip_info(&self, ip: &str) -> Result<Option<HybridIpInfo>> {
        // Skip local/private IPs (no log, just return)
        if self.is_local_or_private_ip(ip) {
            return Ok(None);
        }

        // 1. Try cache first
        if let Some(cached_info) = self.cache_repo.get(ip).await? {
            return Ok(Some(cached_info));
        }

        // 2. Not in cache - query whois/api
        let whois_info = self.whois_repo.get_whois_info(ip).await?;

        let result = match whois_info {
            Some(whois) if whois.country.is_some() => {
                // Whois has country - use it (no need for API call)
                Some(self.from_whois_only(ip, whois))
            }
            Some(whois) => {
                // Whois has NO country - query API for geolocation
                let geo_info = self.geo_repo.get_geolocation(ip).await?;

                match geo_info {
                    Some(geo) => Some(self.from_hybrid(ip, whois, geo)),
                    None => Some(self.from_whois_only(ip, whois)),
                }
            }
            None => {
                // Whois failed - try API only
                let geo_info = self.geo_repo.get_geolocation(ip).await?;

                match geo_info {
                    Some(geo) => Some(self.from_geo_only(ip, geo)),
                    None => None,
                }
            }
        };

        // 3. Save to cache if we got a result
        if let Some(ref info) = result {
            let _ = self.cache_repo.set(ip, info.clone()).await;
        }

        Ok(result)
    }

    /// Create HybridIpInfo from whois only
    fn from_whois_only(&self, ip: &str, whois: WhoisInfo) -> HybridIpInfo {
        HybridIpInfo {
            ip: ip.to_string(),
            country: whois.country.clone().unwrap_or_else(|| "Unknown".to_string()),
            country_code: None,
            city: None,
            organization: whois.organization.unwrap_or_else(|| "Unknown".to_string()),
            isp: None,
            asn: whois.asn,
            ip_range: whois.ip_range,
            source: "whois".to_string(),
        }
    }

    /// Create HybridIpInfo from geo API only
    fn from_geo_only(&self, ip: &str, geo: IpGeolocationInfo) -> HybridIpInfo {
        HybridIpInfo {
            ip: ip.to_string(),
            country: geo.country.clone(),
            country_code: Some(geo.country_code.clone()),
            city: Some(geo.city.clone()),
            organization: geo.isp.clone(),
            isp: Some(geo.isp),
            asn: None,
            ip_range: None,
            source: "api".to_string(),
        }
    }

    /// Create HybridIpInfo from both sources (best of both)
    fn from_hybrid(&self, ip: &str, whois: WhoisInfo, geo: IpGeolocationInfo) -> HybridIpInfo {
        HybridIpInfo {
            ip: ip.to_string(),
            // Prefer geo country (more accurate)
            country: geo.country.clone(),
            country_code: Some(geo.country_code.clone()),
            city: Some(geo.city.clone()),
            // Prefer whois organization (more detailed)
            organization: whois.organization
                .unwrap_or_else(|| geo.isp.clone()),
            isp: Some(geo.isp),
            asn: whois.asn,
            ip_range: whois.ip_range,
            source: "hybrid".to_string(),
        }
    }

    /// Extract IP from "IP:Port" format
    pub fn extract_ip_from_address(&self, address: &str) -> String {
        address
            .split(':')
            .next()
            .unwrap_or(address)
            .to_string()
    }

    /// Check if IP is local/private
    fn is_local_or_private_ip(&self, ip: &str) -> bool {
        // Loopback
        if ip.starts_with("127.") || ip == "::1" || ip.starts_with("::ffff:127.") {
            return true;
        }

        // Private IPv4 (RFC 1918)
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

        // Wildcard
        if ip == "0.0.0.0" || ip == "::" || ip == "*" {
            return true;
        }

        false
    }

    /// Get only country (fast)
    pub async fn get_country(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_ip_info(ip).await? {
            Ok(Some(info.country))
        } else {
            Ok(None)
        }
    }

    /// Get only organization (fast)
    pub async fn get_organization(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_ip_info(ip).await? {
            Ok(Some(info.organization))
        } else {
            Ok(None)
        }
    }
}

impl Default for HybridIpInfoRepository {
    fn default() -> Self {
        Self::new()
    }
}
