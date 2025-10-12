use sqlx::{Pool, Postgres, postgres::PgPoolOptions, Row, Column};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::OnceCell;
use anyhow::{Result, Context};
use serde_json::Value;
use std::collections::HashMap;

use crate::{log_debug, log_warn, log_info};

/// PostgreSQL Connection Pool for high-concurrency scenarios (HTTP API, WebSockets, etc.)
///
/// Usage:
///   - Initialize pool at server startup: PostgresPoolClient::initialize().await
///   - Execute query: let result = PostgresPoolClient::query(sql).await
///   - Execute command: let result = PostgresPoolClient::command(sql).await
///   - Shutdown pool: PostgresPoolClient::shutdown().await
///
/// DO NOT use for ETL/CLI scripts - use PostgresClient singleton instead
pub struct PostgresPoolClient {
    pool: Pool<Postgres>,
    is_initialized: bool,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub last_id: Option<i64>,
    pub affected_rows: u64,
}

static INSTANCE: OnceCell<Arc<Mutex<PostgresPoolClient>>> = OnceCell::new();

impl PostgresPoolClient {
    /// Get the singleton instance
    pub fn instance() -> Arc<Mutex<PostgresPoolClient>> {
        INSTANCE
            .get_or_init(|| {
                Arc::new(Mutex::new(PostgresPoolClient {
                    pool: Self::create_dummy_pool(),
                    is_initialized: false,
                }))
            })
            .clone()
    }

    /// Create a dummy pool (will be replaced on initialize)
    fn create_dummy_pool() -> Pool<Postgres> {
        // This is a placeholder, real pool created in initialize()
        // We can't create a proper pool without async in OnceCell::get_or_init
        unimplemented!("Pool must be initialized with initialize() before use")
    }

    /// Initialize connection pool - MUST be called at server startup
    /// Creates a PostgreSQL connection pool with poolSize connections
    pub async fn initialize() -> Result<()> {
        let instance = Self::instance();
        let mut client = instance.lock().await;

        if client.is_initialized {
            log_warn!("PostgresPoolClient already initialized");
            return Ok(());
        }

        let pool_size: u32 = std::env::var("APP_DB_POOL_SIZE")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        let host = std::env::var("APP_DB_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = std::env::var("APP_DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .unwrap_or(5432);
        let database = std::env::var("APP_DB_NAME")
            .unwrap_or_else(|_| "postgres".to_string());
        let user = std::env::var("APP_DB_USER")
            .unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("APP_DB_PWD")
            .unwrap_or_else(|_| "postgres".to_string());

        log_debug!("Initializing Postgres connection pool (size: {})", pool_size);

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        );

        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(&database_url)
            .await
            .context("Failed to create Postgres connection pool")?;

        // Test connection
        sqlx::query("SELECT 1 as test")
            .fetch_one(&pool)
            .await
            .context("Failed to test Postgres pool connection")?;

        client.pool = pool;
        client.is_initialized = true;

        log_info!("Postgres pool initialized successfully (max connections: {})", pool_size);

        Ok(())
    }

    /// Execute a SELECT query and return rows as Vec<HashMap<String, Value>>
    pub async fn query(sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        let instance = Self::instance();
        let client = instance.lock().await;

        if !client.is_initialized {
            anyhow::bail!("PostgresPoolClient not initialized. Call initialize() first.");
        }

        let rows = sqlx::query(sql)
            .fetch_all(&client.pool)
            .await
            .context("Failed to execute query")?;

        let mut result = Vec::new();
        for row in rows {
            let mut map = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let name = column.name().to_string();
                let value: Value = row.try_get_raw(i)
                    .ok()
                    .and_then(|raw| {
                        // Try to decode as different types
                        if let Ok(v) = row.try_get::<String, _>(i) {
                            Some(Value::String(v))
                        } else if let Ok(v) = row.try_get::<i64, _>(i) {
                            Some(Value::Number(v.into()))
                        } else if let Ok(v) = row.try_get::<i32, _>(i) {
                            Some(Value::Number(v.into()))
                        } else if let Ok(v) = row.try_get::<bool, _>(i) {
                            Some(Value::Bool(v))
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Value::Null);

                map.insert(name, value);
            }
            result.push(map);
        }

        Ok(result)
    }

    /// Execute an INSERT/UPDATE/DELETE command
    /// Returns affected rows and last inserted ID (for INSERT)
    pub async fn command(sql: &str) -> Result<CommandResult> {
        let instance = Self::instance();
        let client = instance.lock().await;

        if !client.is_initialized {
            anyhow::bail!("PostgresPoolClient not initialized. Call initialize() first.");
        }

        // Validate command type
        let cleaned_sql = sql
            .trim()
            .to_lowercase()
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join(" ");

        if !cleaned_sql.starts_with("insert into")
            && !cleaned_sql.starts_with("update")
            && !cleaned_sql.starts_with("delete from")
        {
            anyhow::bail!("PostgresPoolClient::command: Only INSERT, UPDATE, or DELETE are allowed.");
        }

        // For INSERT, try to get the ID
        if cleaned_sql.starts_with("insert into") {
            let result = sqlx::query(sql)
                .fetch_optional(&client.pool)
                .await
                .context("Failed to execute INSERT command")?;

            let last_id = result.and_then(|row| row.try_get::<i64, _>("id").ok());

            return Ok(CommandResult {
                last_id,
                affected_rows: 1, // sqlx doesn't provide rows_affected for fetch_optional
            });
        }

        // For UPDATE/DELETE
        let result = sqlx::query(sql)
            .execute(&client.pool)
            .await
            .context("Failed to execute command")?;

        Ok(CommandResult {
            last_id: None,
            affected_rows: result.rows_affected(),
        })
    }

    /// Get pool statistics for monitoring
    pub async fn get_stats() -> (u32, u32) {
        let instance = Self::instance();
        let client = instance.lock().await;

        if !client.is_initialized {
            return (0, 0);
        }

        let size = client.pool.size();
        let idle = client.pool.num_idle() as u32;

        (size, idle)
    }

    /// Shutdown pool - closes all connections
    /// MUST be called during graceful shutdown
    pub async fn shutdown() -> Result<()> {
        let instance = Self::instance();
        let mut client = instance.lock().await;

        if !client.is_initialized {
            log_warn!("PostgresPoolClient not initialized, nothing to shutdown");
            return Ok(());
        }

        log_debug!("Shutting down Postgres connection pool...");

        client.pool.close().await;
        client.is_initialized = false;

        log_info!("Postgres pool shutdown complete");

        Ok(())
    }
}
