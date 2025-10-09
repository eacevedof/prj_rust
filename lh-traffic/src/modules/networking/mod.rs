pub mod domain;
pub mod application;
pub mod infrastructure;

// Re-exportaciones para facilitar el uso
pub use application::{
    GetLocalNetworkTrafficService,
    GetLocalNetworkTrafficInputDto,
    GotLocalNetworkTrafficDto,
    NetworkTrafficSummaryDto,
    Row,
};
pub use domain::types::NetworkConnection;
pub use domain::enums::ConnectionState;
