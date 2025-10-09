use redis::{Client, Connection, ConnectionLike};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::OnceCell;
use anyhow::Result;

/// Redis client singleton
/// Equivalent to TypeScript's RedisClient
pub struct RedisClient {
    client: Client,
    connection: Arc<Mutex<Option<Connection>>>,
}

static INSTANCE: OnceCell<Arc<RedisClient>> = OnceCell::new();

impl RedisClient {
    /// Get or create the singleton instance
    pub fn instance() -> Result<Arc<RedisClient>> {
        if let Some(instance) = INSTANCE.get() {
            return Ok(instance.clone());
        }

        let client = Self::new()?;
        let arc_client = Arc::new(client);

        match INSTANCE.set(arc_client.clone()) {
            Ok(_) => Ok(arc_client),
            Err(_) => Ok(INSTANCE.get().unwrap().clone()),
        }
    }

    /// Create a new Redis client
    fn new() -> Result<Self> {
        let redis_url = std::env::var("APP_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let client = Client::open(redis_url)?;

        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    /// Get a connection to Redis
    pub async fn get_connection(&self) -> Result<Connection> {
        let mut conn_guard = self.connection.lock().await;

        if let Some(ref mut conn) = *conn_guard {
            if conn.check_connection() {
                return Ok(conn.clone());
            }
        }

        let conn = self.client.get_connection()?;
        *conn_guard = Some(conn.clone());
        Ok(conn)
    }

    /// Get the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
}
