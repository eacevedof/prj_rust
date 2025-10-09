use serde::{Serialize, Deserialize};

/// Representa una conexión de red activa en el sistema
/// Similar a la salida de `netstat` o `ss`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    /// Protocolo (TCP, UDP, etc.)
    pub protocol: String,

    /// Dirección local (IP:Puerto)
    pub local_address: String,

    /// Dirección remota (IP:Puerto)
    pub foreign_address: String,

    /// Estado de la conexión (ESTABLISHED, LISTEN, TIME_WAIT, etc.)
    pub state: String,

    /// PID del proceso dueño de la conexión
    pub pid: Option<u32>,

    /// Nombre del programa/proceso
    pub program_name: Option<String>,
}

impl NetworkConnection {
    pub fn new(
        protocol: String,
        local_address: String,
        foreign_address: String,
        state: String,
    ) -> Self {
        Self {
            protocol,
            local_address,
            foreign_address,
            state,
            pid: None,
            program_name: None,
        }
    }

    pub fn with_process_info(mut self, pid: u32, program_name: String) -> Self {
        self.pid = Some(pid);
        self.program_name = Some(program_name);
        self
    }
}
