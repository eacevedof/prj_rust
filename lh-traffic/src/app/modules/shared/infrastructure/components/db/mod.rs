// ============================================================================
// Database clients - Singleton and Pool versions
// ============================================================================

// PostgreSQL clients
pub mod postgres_client;
pub mod postgres_pool_client;

// Redis clients
pub mod redis_client;
pub mod redis_pool_client;

// Re-exports for easier access
pub use postgres_client::PostgresClient;
pub use postgres_pool_client::{PostgresPoolClient, CommandResult};
pub use redis_client::RedisClient;
pub use redis_pool_client::{RedisPoolClient, PoolStats};
