use redis::{Client, aio::MultiplexedConnection};
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::OnceCell;
use anyhow::{Result, Context};

use crate::{log_debug, log_warn, log_info, log_error};

/// Redis Connection Pool for high-concurrency scenarios (HTTP API, WebSockets, etc.)
///
/// Usage:
///   - Initialize pool at server startup: RedisPoolClient::initialize().await
///   - Acquire connection: let conn = RedisPoolClient::acquire().await
///   - Use connection: conn.get(key).await
///   - Release connection: RedisPoolClient::release(conn).await
///   - Shutdown pool: RedisPoolClient::shutdown().await
///
/// DO NOT use for ETL/CLI scripts - use RedisClient singleton instead
pub struct RedisPoolClient {
    pool: Vec<MultiplexedConnection>,
    available: Vec<bool>,
    pool_size: usize,
    is_initialized: bool,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total: usize,
    pub available: usize,
    pub busy: usize,
}

static INSTANCE: OnceCell<Arc<Mutex<RedisPoolClient>>> = OnceCell::new();

impl RedisPoolClient {
    /// Get the singleton instance
    pub fn instance() -> Arc<Mutex<RedisPoolClient>> {
        INSTANCE
            .get_or_init(|| {
                Arc::new(Mutex::new(RedisPoolClient {
                    pool: Vec::new(),
                    available: Vec::new(),
                    pool_size: 0,
                    is_initialized: false,
                }))
            })
            .clone()
    }

    /// Initialize connection pool - MUST be called at server startup
    /// Creates poolSize persistent connections to Redis
    pub async fn initialize() -> Result<()> {
        let instance = Self::instance();
        let mut client = instance.lock().await;

        if client.is_initialized {
            log_warn!("RedisPoolClient already initialized");
            return Ok(());
        }

        let redis_url = std::env::var("APP_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_db: i64 = std::env::var("APP_REDIS_DB")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);

        let pool_size: usize = std::env::var("APP_REDIS_POOL_SIZE")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        log_debug!("Initializing Redis connection pool (size: {})", pool_size);

        let redis_client = Client::open(redis_url.clone())
            .context("Failed to create Redis client")?;

        let mut pool = Vec::with_capacity(pool_size);
        let mut available = Vec::with_capacity(pool_size);

        for i in 0..pool_size {
            match redis_client.get_multiplexed_async_connection().await {
                Ok(conn) => {
                    pool.push(conn);
                    available.push(true);
                    log_debug!("Redis connection #{} established", i);
                }
                Err(e) => {
                    log_error!("Failed to create Redis connection #{}: {}", i, e);
                    return Err(e.into());
                }
            }
        }

        client.pool = pool;
        client.available = available;
        client.pool_size = pool_size;
        client.is_initialized = true;

        log_info!("Redis pool initialized successfully with {} connections", pool_size);

        Ok(())
    }

    /// Acquire a connection from the pool
    /// Waits if all connections are busy
    pub async fn acquire() -> Result<(usize, MultiplexedConnection)> {
        let instance = Self::instance();

        loop {
            let mut client = instance.lock().await;

            if !client.is_initialized {
                anyhow::bail!("RedisPoolClient not initialized. Call initialize() first.");
            }

            // Find first available connection
            if let Some(idx) = client.available.iter().position(|&v| v) {
                client.available[idx] = false;
                let conn = client.pool[idx].clone();
                return Ok((idx, conn));
            }

            // All connections busy, drop lock and wait
            drop(client);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Release a connection back to the pool
    /// DOES NOT close the connection, just marks it as available
    pub async fn release(idx: usize) -> Result<()> {
        let instance = Self::instance();
        let mut client = instance.lock().await;

        if !client.is_initialized {
            anyhow::bail!("RedisPoolClient not initialized");
        }

        if idx >= client.pool_size {
            anyhow::bail!("Invalid connection index: {}", idx);
        }

        client.available[idx] = true;

        Ok(())
    }

    /// Get pool statistics for monitoring
    pub async fn get_stats() -> PoolStats {
        let instance = Self::instance();
        let client = instance.lock().await;

        if !client.is_initialized {
            return PoolStats {
                total: 0,
                available: 0,
                busy: 0,
            };
        }

        let available_count = client.available.iter().filter(|&&v| v).count();

        PoolStats {
            total: client.pool_size,
            available: available_count,
            busy: client.pool_size - available_count,
        }
    }

    /// Shutdown pool - closes all connections
    /// MUST be called during graceful shutdown
    pub async fn shutdown() -> Result<()> {
        let instance = Self::instance();
        let mut client = instance.lock().await;

        if !client.is_initialized {
            log_warn!("RedisPoolClient not initialized, nothing to shutdown");
            return Ok(());
        }

        log_debug!("Shutting down Redis connection pool...");

        // Clear the pool (connections will be dropped)
        client.pool.clear();
        client.available.clear();
        client.pool_size = 0;
        client.is_initialized = false;

        log_info!("Redis pool shutdown complete");

        Ok(())
    }
}
