use redis::{Client, aio::MultiplexedConnection};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::OnceCell;
use anyhow::Result;

use crate::log_debug;

/// Redis client singleton (NO pool)
///
/// For CLI/ETL scripts that don't need connection pooling.
/// Uses a single multiplexed connection that can be reused.
///
/// For HTTP APIs / high-concurrency scenarios, use RedisPoolClient instead.
///
/// Equivalent to TypeScript's RedisClient
pub struct RedisClient {
    client: Client,
    connection: Arc<Mutex<Option<MultiplexedConnection>>>,
}

static INSTANCE: OnceCell<Arc<RedisClient>> = OnceCell::new();

impl RedisClient {
    /// Get or create the singleton instance
    pub fn instance() -> Arc<RedisClient> {
        INSTANCE
            .get_or_init(|| {
                let client = Self::new().expect("Failed to create RedisClient");
                Arc::new(client)
            })
            .clone()
    }

    /// Create a new Redis client
    fn new() -> Result<Self> {
        let redis_url = std::env::var("APP_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_db: i64 = std::env::var("APP_REDIS_DB")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);

        log_debug!("RedisClient initialized with URL: {}, DB: {}", redis_url, redis_db);

        let client = Client::open(redis_url)?;

        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    /// Get a connection to Redis (creates one if not exists)
    /// Returns a multiplexed connection that can be reused
    pub async fn get_connection(&self) -> Result<MultiplexedConnection> {
        let mut conn_guard = self.connection.lock().await;

        if conn_guard.is_none() {
            let conn = self.client.get_multiplexed_async_connection().await?;
            *conn_guard = Some(conn);
        }

        Ok(conn_guard.as_ref().unwrap().clone())
    }

    /// Get the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Close the connection (if open)
    pub async fn close(&self) {
        let mut conn_guard = self.connection.lock().await;
        *conn_guard = None;
        log_debug!("RedisClient connection closed");
    }
}
