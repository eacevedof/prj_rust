use std::process::Command;
use anyhow::{Result, Context};
use std::sync::Arc;

use crate::app::modules::shared::infrastructure::components::logger::Logger;

/// Información de un proceso
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// PID del proceso
    pub pid: u32,

    /// Nombre del proceso (ej: "firefox.exe")
    pub name: String,

    /// Path completo del ejecutable
    pub path: String,

    /// Directorio de trabajo
    pub working_directory: Option<String>,

    /// Usuario propietario del proceso (Windows: DOMAIN\User)
    pub owner: Option<String>,
}

/// Repositorio para obtener información de procesos del sistema
/// En Windows usa PowerShell Get-Process
/// En Linux usa /proc filesystem
pub struct ProcessInfoRepository {
    logger: Arc<Logger>,
}

impl ProcessInfoRepository {
    pub fn new() -> Self {
        Self {
            logger: Logger::instance(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Obtiene información completa de un proceso por su PID
    ///
    /// # Arguments
    /// * `pid` - Process ID
    ///
    /// # Returns
    /// * `Ok(Some(ProcessInfo))` - Si se encontró el proceso
    /// * `Ok(None)` - Si el proceso no existe o no se pudo obtener información
    /// * `Err` - Si hubo un error en la consulta
    pub async fn get_process_info(&self, pid: u32) -> Result<Option<ProcessInfo>> {
        self.logger
            .log_debug(
                &format!("Getting process info for PID: {}", pid),
                "ProcessInfoRepository"
            )
            .await;

        if cfg!(target_os = "windows") {
            self.get_process_info_windows(pid).await
        } else {
            self.get_process_info_linux(pid).await
        }
    }

    /// Obtiene información del proceso en Windows usando PowerShell
    async fn get_process_info_windows(&self, pid: u32) -> Result<Option<ProcessInfo>> {
        // PowerShell command para obtener información completa del proceso
        // Get-Process -Id PID | Select-Object Id, Name, Path, @{Name='Owner';Expression={(Get-WmiObject -Class Win32_Process -Filter \"ProcessId=$($_.Id)\").GetOwner().User}}
        let ps_script = format!(
            r#"
            $p = Get-Process -Id {} -ErrorAction SilentlyContinue
            if ($p) {{
                $owner = (Get-WmiObject -Class Win32_Process -Filter "ProcessId=$($p.Id)" -ErrorAction SilentlyContinue)
                if ($owner) {{
                    $ownerInfo = $owner.GetOwner()
                    $ownerName = "$($ownerInfo.Domain)\$($ownerInfo.User)"
                }} else {{
                    $ownerName = ""
                }}
                Write-Output "$($p.Id)|$($p.Name)|$($p.Path)|$ownerName"
            }}
            "#,
            pid
        );

        let output = Command::new("powershell.exe")
            .args(&[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &ps_script
            ])
            .output()
            .context("Failed to execute PowerShell command")?;

        if !output.status.success() {
            self.logger
                .log_debug(
                    &format!("PowerShell command failed for PID {}", pid),
                    "ProcessInfoRepository"
                )
                .await;
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();

        if line.is_empty() {
            return Ok(None);
        }

        self.parse_windows_output(line)
    }

    /// Parsea la salida de PowerShell
    /// Formato: "PID|Name|Path|Owner"
    fn parse_windows_output(&self, line: &str) -> Result<Option<ProcessInfo>> {
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() < 3 {
            return Ok(None);
        }

        let pid = parts[0].parse::<u32>().ok();
        let name = parts[1].to_string();
        let path = parts[2].to_string();
        let owner = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].to_string())
        } else {
            None
        };

        if let Some(pid) = pid {
            Ok(Some(ProcessInfo {
                pid,
                name: if name.is_empty() { format!("PID:{}", pid) } else { name },
                path: if path.is_empty() || path == "null" {
                    "Unknown".to_string()
                } else {
                    path
                },
                working_directory: None, // No lo obtenemos por simplicidad
                owner,
            }))
        } else {
            Ok(None)
        }
    }

    /// Obtiene información del proceso en Linux leyendo /proc
    async fn get_process_info_linux(&self, pid: u32) -> Result<Option<ProcessInfo>> {
        let proc_path = format!("/proc/{}", pid);
        let exe_path = format!("{}/exe", proc_path);
        let _cmdline_path = format!("{}/cmdline", proc_path);

        // Verificar que el proceso existe
        if !std::path::Path::new(&proc_path).exists() {
            return Ok(None);
        }

        // Obtener el path del ejecutable (symlink)
        let path = match std::fs::read_link(&exe_path) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => "Unknown".to_string(),
        };

        // Obtener el nombre del proceso
        let name = path
            .split('/')
            .last()
            .unwrap_or(&format!("PID:{}", pid))
            .to_string();

        // Obtener el owner (usuario)
        let owner = self.get_process_owner_linux(pid).await.ok();

        Ok(Some(ProcessInfo {
            pid,
            name,
            path,
            working_directory: None,
            owner,
        }))
    }

    /// Obtiene el propietario de un proceso en Linux
    async fn get_process_owner_linux(&self, pid: u32) -> Result<String> {
        let status_path = format!("/proc/{}/status", pid);
        let content = std::fs::read_to_string(&status_path)?;

        // Buscar la línea "Uid:"
        for line in content.lines() {
            if line.starts_with("Uid:") {
                if let Some(uid_str) = line.split_whitespace().nth(1) {
                    return Ok(format!("UID:{}", uid_str));
                }
            }
        }

        Ok("Unknown".to_string())
    }

    /// Obtiene solo el path del proceso (método rápido)
    pub async fn get_process_path(&self, pid: u32) -> Result<Option<String>> {
        if let Some(info) = self.get_process_info(pid).await? {
            Ok(Some(info.path))
        } else {
            Ok(None)
        }
    }

    /// Obtiene solo el nombre del proceso (método rápido)
    pub async fn get_process_name(&self, pid: u32) -> Result<Option<String>> {
        if let Some(info) = self.get_process_info(pid).await? {
            Ok(Some(info.name))
        } else {
            Ok(None)
        }
    }

    /// Verifica si un proceso existe
    pub async fn process_exists(&self, pid: u32) -> bool {
        if cfg!(target_os = "windows") {
            // En Windows, usar tasklist o PowerShell
            let output = Command::new("powershell.exe")
                .args(&[
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    &format!("Get-Process -Id {} -ErrorAction SilentlyContinue", pid)
                ])
                .output();

            output.map(|o| o.status.success()).unwrap_or(false)
        } else {
            // En Linux, verificar /proc/{pid}
            std::path::Path::new(&format!("/proc/{}", pid)).exists()
        }
    }
}

impl Default for ProcessInfoRepository {
    fn default() -> Self {
        Self::new()
    }
}
