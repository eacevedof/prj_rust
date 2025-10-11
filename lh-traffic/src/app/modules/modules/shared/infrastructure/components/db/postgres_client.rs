use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::sync::Arc;
use once_cell::sync::OnceCell;
use anyhow::Result;

/// PostgreSQL client singleton
/// Equivalent to TypeScript's PostgresClient
pub struct PostgresClient {
    pool: Pool<Postgres>,
}

static INSTANCE: OnceCell<Arc<PostgresClient>> = OnceCell::new();

impl PostgresClient {
    /// Get or create the singleton instance
    pub async fn instance() -> Result<Arc<PostgresClient>> {
        if let Some(instance) = INSTANCE.get() {
            return Ok(instance.clone());
        }

        let client = Self::new().await?;
        let arc_client = Arc::new(client);

        // Try to set the instance, if it fails someone else already set it
        match INSTANCE.set(arc_client.clone()) {
            Ok(_) => Ok(arc_client),
            Err(_) => Ok(INSTANCE.get().unwrap().clone()),
        }
    }

    /// Create a new PostgreSQL client
    async fn new() -> Result<Self> {
        let database_url = Self::get_connection_string();

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Get connection string from environment
    fn get_connection_string() -> String {
        let host = std::env::var("APP_DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("APP_DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let database = std::env::var("APP_DB_NAME").unwrap_or_else(|_| "postgres".to_string());
        let user = std::env::var("APP_DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("APP_DB_PWD").unwrap_or_else(|_| "postgres".to_string());

        format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        )
    }

    /// Close all connections
    pub async fn close(&self) {
        self.pool.close().await;
    }
}
