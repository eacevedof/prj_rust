use anyhow::Result;
use std::collections::HashMap;
use redis::AsyncCommands;

use crate::app::modules::shared::infrastructure::components::db::{
    RedisClient, RedisPoolClient
};

const SIXTY_SECONDS: i64 = 60;

/// Abstract base repository for Redis operations
///
/// Provides both singleton (CLI/ETL) and pool-based (HTTP API) methods.
/// Supports hash sets, queues, and bulk operations with pipelines.
///
/// Usage in Rust:
/// ```rust
/// pub struct MyCacheRepository {
///     base: AbstractRedisRepository,
/// }
///
/// impl MyCacheRepository {
///     pub async fn get_user_cache(&self, user_id: &str) -> Result<Option<HashMap<String, String>>> {
///         let redis_key = format!("user:{}", user_id);
///         self.base.get_hash_set(&redis_key).await
///     }
/// }
/// ```
pub struct AbstractRedisRepository {
    _environment: String,
}

impl AbstractRedisRepository {
    pub fn new() -> Self {
        let _environment = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string());

        Self { _environment }
    }

    // ========================================================================
    // SIMPLE METHODS (for CLI/ETL - creates connection each time)
    // ========================================================================

    /// Get hash set from Redis
    /// Returns None if key doesn't exist or is empty
    pub async fn get_hash_set(&self, redis_key: &str) -> Result<Option<HashMap<String, String>>> {
        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        let result: HashMap<String, String> = conn.hgetall(redis_key).await?;

        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    /// Get hash set for bulk operations (connection managed externally)
    /// Note: In Rust, this is the same as get_hash_set since we use multiplexed connections
    pub async fn get_hash_set_for_bulk(&self, redis_key: &str) -> Result<Option<HashMap<String, String>>> {
        self.get_hash_set(redis_key).await
    }

    /// Save a single hash set with TTL
    pub async fn save_single_hash_set(
        &self,
        redis_key: &str,
        data: &HashMap<String, String>,
        ttl_minutes: i64,
    ) -> Result<()> {
        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        // Set hash
        for (field, value) in data {
            conn.hset::<_, _, _, ()>(redis_key, field, value).await?;
        }

        // Set expiration
        conn.expire::<_, ()>(redis_key, (ttl_minutes * SIXTY_SECONDS) as i64).await?;

        Ok(())
    }

    /// Save to a Redis queue (LPUSH)
    pub async fn save_in_queue(
        &self,
        queue_name: &str,
        data: &HashMap<String, String>,
    ) -> Result<()> {
        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        let json_data = serde_json::to_string(data)?;

        conn.lpush::<_, _, ()>(queue_name, json_data).await?;

        Ok(())
    }

    /// Delete a single key
    pub async fn delete_single_key(&self, redis_key: &str) -> Result<()> {
        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        conn.del::<_, ()>(redis_key).await?;

        Ok(())
    }

    /// Save single hash set for bulk (same as save_single_hash_set in Rust)
    pub async fn save_single_hash_set_for_bulk(
        &self,
        redis_key: &str,
        data: &HashMap<String, String>,
        ttl_minutes: i64,
    ) -> Result<()> {
        self.save_single_hash_set(redis_key, data, ttl_minutes).await
    }

    // ========================================================================
    // BULK OPERATIONS (using Redis pipelines)
    // ========================================================================

    /// Get multiple hash sets using Redis pipeline
    /// Connection must be provided (for reuse in bulk operations)
    pub async fn get_bulk_hash_sets(
        &self,
        redis_keys: &[String],
    ) -> Result<Vec<HashMap<String, String>>> {
        if redis_keys.is_empty() {
            return Ok(Vec::new());
        }

        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        let mut pipe = redis::pipe();

        for key in redis_keys {
            pipe.hgetall(key);
        }

        let results: Vec<HashMap<String, String>> = pipe.query_async(&mut conn).await?;

        // Filter out empty hash sets
        let non_empty: Vec<HashMap<String, String>> = results
            .into_iter()
            .filter(|map| !map.is_empty())
            .collect();

        Ok(non_empty)
    }

    /// Save multiple hash sets using Redis pipeline
    /// Connection must be opened before calling this method
    pub async fn save_bulk_hash_sets(
        &self,
        operations: &[BulkHashSetOperation],
    ) -> Result<()> {
        if operations.is_empty() {
            return Ok(());
        }

        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        let mut pipe = redis::pipe();

        for op in operations {
            for (field, value) in &op.redis_row {
                pipe.hset(&op.redis_key, field, value);
            }
            pipe.expire(&op.redis_key, (op.redis_ttl_mins * SIXTY_SECONDS) as i64);
        }

        pipe.query_async::<_, ()>(&mut conn).await?;

        Ok(())
    }

    // ========================================================================
    // POOL-BASED METHODS (for HTTP API - uses connection pool)
    // ========================================================================

    /// Get hash set using connection pool
    pub async fn get_hash_set_pool(&self, redis_key: &str) -> Result<Option<HashMap<String, String>>> {
        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        let result: HashMap<String, String> = conn.hgetall(redis_key).await?;

        RedisPoolClient::release(idx).await?;

        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    /// Save single hash set using connection pool
    pub async fn save_single_hash_set_pool(
        &self,
        redis_key: &str,
        data: &HashMap<String, String>,
        ttl_minutes: i64,
    ) -> Result<()> {
        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        // Set hash
        for (field, value) in data {
            conn.hset::<_, _, _, ()>(redis_key, field, value).await?;
        }

        // Set expiration
        conn.expire::<_, ()>(redis_key, (ttl_minutes * SIXTY_SECONDS) as i64).await?;

        RedisPoolClient::release(idx).await?;

        Ok(())
    }

    /// Delete single key using connection pool
    pub async fn delete_single_key_pool(&self, redis_key: &str) -> Result<()> {
        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        conn.del::<_, ()>(redis_key).await?;

        RedisPoolClient::release(idx).await?;

        Ok(())
    }

    /// Get bulk hash sets using connection pool with pipeline
    pub async fn get_bulk_hash_sets_pool(
        &self,
        redis_keys: &[String],
    ) -> Result<Vec<HashMap<String, String>>> {
        if redis_keys.is_empty() {
            return Ok(Vec::new());
        }

        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        let mut pipe = redis::pipe();

        for key in redis_keys {
            pipe.hgetall(key);
        }

        let results: Vec<HashMap<String, String>> = pipe.query_async(&mut conn).await?;

        RedisPoolClient::release(idx).await?;

        // Filter out empty hash sets
        let non_empty: Vec<HashMap<String, String>> = results
            .into_iter()
            .filter(|map| !map.is_empty())
            .collect();

        Ok(non_empty)
    }

    /// Save bulk hash sets using connection pool with pipeline
    pub async fn save_bulk_hash_sets_pool(
        &self,
        operations: &[BulkHashSetOperation],
    ) -> Result<()> {
        if operations.is_empty() {
            return Ok(());
        }

        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        let mut pipe = redis::pipe();

        for op in operations {
            for (field, value) in &op.redis_row {
                pipe.hset(&op.redis_key, field, value);
            }
            pipe.expire(&op.redis_key, (op.redis_ttl_mins * SIXTY_SECONDS) as i64);
        }

        pipe.query_async::<_, ()>(&mut conn).await?;

        RedisPoolClient::release(idx).await?;

        Ok(())
    }

    /// Push to queue using connection pool
    /// Example: self.save_in_queue_pool("email:queue", &data).await
    pub async fn save_in_queue_pool(
        &self,
        queue_name: &str,
        data: &HashMap<String, String>,
    ) -> Result<()> {
        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        let json_data = serde_json::to_string(data)?;

        conn.lpush::<_, _, ()>(queue_name, json_data).await?;

        RedisPoolClient::release(idx).await?;

        Ok(())
    }
}

impl Default for AbstractRedisRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Structure for bulk hash set operations
#[derive(Debug, Clone)]
pub struct BulkHashSetOperation {
    pub redis_key: String,
    pub redis_row: HashMap<String, String>,
    pub redis_ttl_mins: i64,
}

impl BulkHashSetOperation {
    pub fn new(redis_key: String, redis_row: HashMap<String, String>, redis_ttl_mins: i64) -> Self {
        Self {
            redis_key,
            redis_row,
            redis_ttl_mins,
        }
    }
}
