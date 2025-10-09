pub mod domain;
pub mod application;
pub mod infrastructure;

// Re-exportaciones para facilitar el uso
pub use application::services::GetLocalNetworkTrafficService;
pub use domain::types::NetworkConnection;
pub use domain::enums::ConnectionState;
