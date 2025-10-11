/// DTO con el resumen del tráfico de red
#[derive(Debug, Clone)]
pub struct NetworkTrafficSummaryDto {
    pub total_connections: usize,
    pub tcp_connections: usize,
    pub udp_connections: usize,
    pub established_connections: usize,
    pub listening_connections: usize,
}

impl NetworkTrafficSummaryDto {
    pub fn new(
        total_connections: usize,
        tcp_connections: usize,
        udp_connections: usize,
        established_connections: usize,
        listening_connections: usize,
    ) -> Self {
        Self {
            total_connections,
            tcp_connections,
            udp_connections,
            established_connections,
            listening_connections,
        }
    }

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
