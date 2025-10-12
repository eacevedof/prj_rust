use sqlx::{postgres::PgConnection, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::OnceCell;
use anyhow::Result;

use crate::log_debug;

/// PostgreSQL client singleton (NO pool)
///
/// For CLI/ETL scripts that don't need connection pooling.
/// Creates one connection per operation, opening and closing it each time.
///
/// For HTTP APIs / high-concurrency scenarios, use PostgresPoolClient instead.
///
/// Equivalent to TypeScript's PostgresClient
pub struct PostgresClient {
    config: PostgresConfig,
}

#[derive(Clone)]
struct PostgresConfig {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

static INSTANCE: OnceCell<Arc<Mutex<PostgresClient>>> = OnceCell::new();

impl PostgresClient {
    /// Get or create the singleton instance
    pub fn instance() -> Arc<Mutex<PostgresClient>> {
        INSTANCE
            .get_or_init(|| {
                let client = Self::new();
                Arc::new(Mutex::new(client))
            })
            .clone()
    }

    /// Create a new PostgreSQL client (private)
    fn new() -> Self {
        let config = Self::load_config_from_env();

        log_debug!(
            "PostgresClient initialized with host: {}:{}",
            config.host,
            config.port
        );

        Self { config }
    }

    /// Load configuration from environment variables
    fn load_config_from_env() -> PostgresConfig {
        PostgresConfig {
            host: std::env::var("APP_DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("APP_DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: std::env::var("APP_DB_NAME").unwrap_or_else(|_| "postgres".to_string()),
            user: std::env::var("APP_DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("APP_DB_PWD").unwrap_or_else(|_| "postgres".to_string()),
        }
    }

    /// Get connection string
    fn get_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.config.user,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.database
        )
    }

    /// Get a new connection (opens a new connection each time)
    /// This matches Deno's behavior where each query opens/closes a connection
    pub async fn get_connection(&self) -> Result<PgConnection> {
        let conn_str = self.get_connection_string();
        let conn = PgConnection::connect(&conn_str).await?;
        Ok(conn)
    }

    /// Close all connections (in this implementation, there's nothing to close)
    /// Kept for API compatibility with Deno version
    pub async fn close_all_connections() {
        log_debug!("PostgresClient: close_all_connections called (no-op for singleton)");
    }
}
