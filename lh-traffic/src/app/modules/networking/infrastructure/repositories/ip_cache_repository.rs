use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use super::{HybridIpInfo};

/// Entrada de caché con información de IP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedIpInfo {
    /// IP address
    pub ip: String,

    /// Información de la IP
    pub data: HybridIpInfo,

    /// Fecha de creación del caché
    pub created_at: DateTime<Utc>,

    /// Fecha de expiración del caché
    pub expires_at: DateTime<Utc>,
}

/// Repositorio para gestionar el caché de información de IPs en JSON
///
/// Características:
/// - Un archivo JSON por IP
/// - Expiración: 15 días
/// - Ubicación: cache/ip_info/
pub struct IpCacheRepository {
    cache_dir: PathBuf,
    ttl_days: i64,
}

impl IpCacheRepository {
    pub fn new() -> Self {
        Self {
            cache_dir: PathBuf::from("cache/ip_info"),
            ttl_days: 15,
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Obtiene información de IP del caché si existe y no ha expirado
    ///
    /// # Arguments
    /// * `ip` - Dirección IP
    ///
    /// # Returns
    /// * `Ok(Some(HybridIpInfo))` - Si existe en caché y no ha expirado
    /// * `Ok(None)` - Si no existe o ha expirado
    /// * `Err` - Si hubo un error leyendo el archivo
    pub async fn get(&self, ip: &str) -> Result<Option<HybridIpInfo>> {
        let cache_path = self.get_cache_path(ip);

        // Verificar si existe el archivo
        if !cache_path.exists() {
            return Ok(None);
        }

        // Leer el archivo
        let content = fs::read_to_string(&cache_path)
            .context("Failed to read cache file")?;

        // Parsear JSON
        let cached: CachedIpInfo = serde_json::from_str(&content)
            .context("Failed to parse cache JSON")?;

        // Verificar si ha expirado
        if Utc::now() > cached.expires_at {
            // Expirado - borrar el archivo
            let _ = fs::remove_file(&cache_path);
            return Ok(None);
        }

        Ok(Some(cached.data))
    }

    /// Guarda información de IP en el caché
    ///
    /// # Arguments
    /// * `ip` - Dirección IP
    /// * `info` - Información de la IP
    ///
    /// # Returns
    /// * `Ok(())` - Si se guardó correctamente
    /// * `Err` - Si hubo un error escribiendo el archivo
    pub async fn set(&self, ip: &str, info: HybridIpInfo) -> Result<()> {
        // Crear directorio de caché si no existe
        fs::create_dir_all(&self.cache_dir)
            .context("Failed to create cache directory")?;

        let now = Utc::now();
        let expires_at = now + Duration::days(self.ttl_days);

        let cached = CachedIpInfo {
            ip: ip.to_string(),
            data: info,
            created_at: now,
            expires_at,
        };

        // Serializar a JSON
        let json = serde_json::to_string_pretty(&cached)
            .context("Failed to serialize cache to JSON")?;

        // Escribir al archivo
        let cache_path = self.get_cache_path(ip);
        fs::write(&cache_path, json)
            .context("Failed to write cache file")?;

        Ok(())
    }

    /// Verifica si existe información en caché para una IP (sin verificar expiración)
    pub async fn exists(&self, ip: &str) -> bool {
        self.get_cache_path(ip).exists()
    }

    /// Elimina información de IP del caché
    pub async fn delete(&self, ip: &str) -> Result<()> {
        let cache_path = self.get_cache_path(ip);
        if cache_path.exists() {
            fs::remove_file(&cache_path)
                .context("Failed to delete cache file")?;
        }
        Ok(())
    }

    /// Limpia todas las entradas expiradas del caché
    pub async fn clean_expired(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut cleaned_count = 0;

        let entries = fs::read_dir(&self.cache_dir)
            .context("Failed to read cache directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Leer y verificar expiración
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cached) = serde_json::from_str::<CachedIpInfo>(&content) {
                    if Utc::now() > cached.expires_at {
                        let _ = fs::remove_file(&path);
                        cleaned_count += 1;
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Obtiene la ruta del archivo de caché para una IP
    fn get_cache_path(&self, ip: &str) -> PathBuf {
        // Sanitizar el nombre del archivo (reemplazar : por _ para IPv6)
        let safe_filename = ip.replace(':', "_");
        self.cache_dir.join(format!("{}.json", safe_filename))
    }

    /// Obtiene el tamaño total del caché en bytes
    pub async fn get_cache_size(&self) -> Result<u64> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;

        let entries = fs::read_dir(&self.cache_dir)
            .context("Failed to read cache directory")?;

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// Obtiene el número de entradas en caché
    pub async fn get_cache_count(&self) -> Result<usize> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let count = fs::read_dir(&self.cache_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .count();

        Ok(count)
    }
}

impl Default for IpCacheRepository {
    fn default() -> Self {
        Self::new()
    }
}
