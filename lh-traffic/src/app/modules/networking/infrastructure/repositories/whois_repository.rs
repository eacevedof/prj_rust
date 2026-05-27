use std::process::Command;
use anyhow::{Result, Context};


/// Información obtenida de whois
#[derive(Debug, Clone)]
pub struct WhoisInfo {
    /// IP consultada
    pub ip: String,

    /// Organización propietaria
    pub organization: Option<String>,

    /// País (puede no estar disponible)
    pub country: Option<String>,

    /// Nombre de la red/ISP
    pub net_name: Option<String>,

    /// Rango de IPs
    pub ip_range: Option<String>,

    /// ASN (Autonomous System Number)
    pub asn: Option<String>,

    /// Descripción del ASN
    pub asn_description: Option<String>,

    /// Output completo de whois (por si necesitas parsearlo más)
    pub raw_output: String,
}

/// Repositorio para obtener información usando el comando whois
///
/// Ventajas:
/// - Sin límite de rate (cada consulta es independiente)
/// - Información del propietario de la IP
/// - ASN y rango de IPs
///
/// Desventajas:
/// - No siempre incluye país/ciudad
/// - Más lento que APIs (2-5 segundos)
/// - Formato de salida varía según el servidor whois
/// - Requiere comando whois instalado
pub struct WhoisRepository;

impl WhoisRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Obtiene información whois para una IP
    ///
    /// # Arguments
    /// * `ip` - Dirección IP (ej: "8.8.8.8")
    ///
    /// # Returns
    /// * `Ok(Some(WhoisInfo))` - Si se encontró información
    /// * `Ok(None)` - Si la IP es local/privada o no se pudo obtener info
    /// * `Err` - Si el comando whois no está disponible o falló
    pub async fn get_whois_info(&self, ip: &str) -> Result<Option<WhoisInfo>> {
        // Filtrar IPs locales/privadas (sin log)
        if self.is_local_or_private_ip(ip) {
            return Ok(None);
        }

        // Ejecutar comando whois
        let output = if cfg!(target_os = "windows") {
            // En Windows, whois puede no estar instalado por defecto
            // Intentar usar whois.exe si está en el PATH
            Command::new("whois.exe")
                .arg(ip)
                .output()
                .or_else(|_| {
                    // Si no existe whois.exe, intentar con PowerShell
                    Command::new("powershell.exe")
                        .args(&[
                            "-NoProfile",
                            "-Command",
                            &format!("(Invoke-WebRequest -Uri 'https://whois.arin.net/rest/ip/{}' -Headers @{{'Accept'='text/plain'}}).Content", ip)
                        ])
                        .output()
                })
                .context("whois command not found. Install whois or use ip-api.com instead")?
        } else {
            // En Linux/Mac, whois normalmente está instalado
            Command::new("whois")
                .arg(ip)
                .output()
                .context("whois command not found. Install with: apt install whois")?
        };

        if !output.status.success() {
            return Ok(None);
        }

        let raw_output = String::from_utf8_lossy(&output.stdout).to_string();

        if raw_output.trim().is_empty() {
            return Ok(None);
        }

        // Parsear la salida de whois
        let info = self.parse_whois_output(ip, &raw_output);

        Ok(Some(info))
    }

    /// Parsea la salida de whois
    /// El formato varía según el servidor whois (ARIN, RIPE, APNIC, etc.)
    fn parse_whois_output(&self, ip: &str, output: &str) -> WhoisInfo {
        let mut organization = None;
        let mut country = None;
        let mut net_name = None;
        let mut ip_range = None;
        let mut asn = None;
        let mut asn_description = None;

        for line in output.lines() {
            let line = line.trim();
            let lower_line = line.to_lowercase();

            // Organization / OrgName
            if organization.is_none() {
                if let Some(value) = self.extract_value(line, &["organization:", "orgname:", "org-name:", "owner:"]) {
                    organization = Some(value);
                }
            }

            // Country
            if country.is_none() {
                if let Some(value) = self.extract_value(line, &["country:", "country code:"]) {
                    country = Some(value);
                }
            }

            // NetName
            if net_name.is_none() {
                if let Some(value) = self.extract_value(line, &["netname:", "net-name:"]) {
                    net_name = Some(value);
                }
            }

            // IP Range
            if ip_range.is_none() {
                if let Some(value) = self.extract_value(line, &["netrange:", "inetnum:", "inet6num:", "cidr:"]) {
                    ip_range = Some(value);
                }
            }

            // ASN
            if asn.is_none() {
                if let Some(value) = self.extract_value(line, &["origin:", "originas:", "asn:"]) {
                    asn = Some(value);
                }
            }

            // ASN Description
            if asn_description.is_none() && lower_line.contains("as ") && lower_line.contains("description") {
                if let Some(value) = self.extract_value(line, &["descr:", "description:"]) {
                    asn_description = Some(value);
                }
            }
        }

        WhoisInfo {
            ip: ip.to_string(),
            organization,
            country,
            net_name,
            ip_range,
            asn,
            asn_description,
            raw_output: output.to_string(),
        }
    }

    /// Extrae el valor después de una clave en una línea
    /// Ej: "Organization: Google LLC" -> Some("Google LLC")
    fn extract_value(&self, line: &str, keys: &[&str]) -> Option<String> {
        let lower_line = line.to_lowercase();

        for key in keys {
            if lower_line.starts_with(key) {
                let value = line[key.len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }

        None
    }

    /// Extrae solo la IP de un string "IP:Puerto"
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

    /// Obtiene solo la organización (más rápido si solo necesitas eso)
    pub async fn get_organization(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_whois_info(ip).await? {
            Ok(info.organization)
        } else {
            Ok(None)
        }
    }

    /// Obtiene solo el país
    pub async fn get_country(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_whois_info(ip).await? {
            Ok(info.country)
        } else {
            Ok(None)
        }
    }

    /// Obtiene el ASN (Autonomous System Number)
    pub async fn get_asn(&self, ip: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_whois_info(ip).await? {
            Ok(info.asn)
        } else {
            Ok(None)
        }
    }
}

impl Default for WhoisRepository {
    fn default() -> Self {
        Self::new()
    }
}
