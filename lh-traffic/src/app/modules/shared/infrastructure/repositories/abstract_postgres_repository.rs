use anyhow::{Result, Context};
use serde_json::Value;
use std::collections::HashMap;
use sqlx::{Row, Column, Connection};
use md5;

use crate::log_debug;

use crate::app::modules::shared::infrastructure::components::db::{
    PostgresClient, PostgresPoolClient
};
use crate::app::modules::shared::infrastructure::components::db::redis_client::RedisClient;
use crate::app::modules::shared::infrastructure::components::db::redis_pool_client::RedisPoolClient;

/// Abstract base repository for PostgreSQL operations
///
/// Provides both singleton (CLI/ETL) and pool-based (HTTP API) methods.
///
/// Usage in Rust:
/// ```rust
/// pub struct MyRepository {
///     base: AbstractPostgresRepository,
/// }
///
/// impl MyRepository {
///     pub async fn find_by_id(&self, id: i64) -> Result<Option<MyEntity>> {
///         let sql = format!("SELECT * FROM my_table WHERE id = {}", id);
///         let rows = self.base.query(&sql).await?;
///         // ... process rows
///         Ok(None)
///     }
/// }
/// ```
pub struct AbstractPostgresRepository {
    pub last_id: Option<i64>,
    pub affected_rows: u64,
    environment: String,
}

impl AbstractPostgresRepository {
    pub fn new() -> Self {
        let environment = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string());

        Self {
            last_id: None,
            affected_rows: 0,
            environment,
        }
    }

    // ========================================================================
    // SIMPLE METHODS (for CLI/ETL - opens and closes connection each time)
    // ========================================================================

    /// Execute a SELECT query (opens/closes connection)
    /// For CLI/ETL use only
    pub async fn query(&mut self, sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        let client_instance = PostgresClient::instance();
        let client = client_instance.lock().await;

        let mut conn = client.get_connection().await?;

        let rows = sqlx::query(sql)
            .fetch_all(&mut conn)
            .await
            .context("Failed to execute query")?;

        conn.close().await?;

        Ok(Self::rows_to_hashmap(rows))
    }

    /// Execute SELECT query with Redis caching
    /// For CLI/ETL use only
    pub async fn query_redis(
        &mut self,
        sql: &str,
        ttl_minutes: i64,
    ) -> Result<Vec<HashMap<String, Value>>> {
        // Try to get from Redis first
        if let Ok(cached) = self.get_from_redis(sql).await {
            if let Some(data) = cached {
                return Ok(serde_json::from_str(&data)?);
            }
        }

        self.log_sql(sql, "query_redis");

        // Not in cache, get from Postgres
        let rows = self.query(sql).await?;

        // Save to Redis cache
        if !rows.is_empty() {
            let _ = self.save_to_redis(sql, &rows, ttl_minutes).await;
        }

        Ok(rows)
    }

    /// Execute INSERT/UPDATE/DELETE command
    /// Updates last_id and affected_rows
    /// For CLI/ETL use only
    pub async fn command(&mut self, sql: &str) -> Result<()> {
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
            anyhow::bail!("AbstractPostgresRepository::command: Only INSERT, UPDATE, or DELETE are allowed");
        }

        let client_instance = PostgresClient::instance();
        let client = client_instance.lock().await;

        let mut conn = client.get_connection().await?;

        // For INSERT, try to get the ID
        if cleaned_sql.starts_with("insert into") {
            let result = sqlx::query(sql)
                .fetch_optional(&mut conn)
                .await
                .context("Failed to execute INSERT command")?;

            self.last_id = result.and_then(|row| row.try_get::<i64, _>("id").ok());
            self.affected_rows = 1;

            conn.close().await?;
            return Ok(());
        }

        // For UPDATE/DELETE
        let result = sqlx::query(sql)
            .execute(&mut conn)
            .await
            .context("Failed to execute command")?;

        self.affected_rows = result.rows_affected();
        self.last_id = None;

        conn.close().await?;

        Ok(())
    }

    // ========================================================================
    // POOL-BASED METHODS (for HTTP API - uses connection pool)
    // ========================================================================

    /// Execute SELECT query using connection pool
    /// For HTTP API use - does not use redis cache
    pub async fn query_pool(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        self.log_sql(sql, "query_pool");
        PostgresPoolClient::query(sql).await
    }

    /// Execute SELECT query with Redis cache using connection pool
    /// For HTTP API use - recommended for frequently accessed data
    pub async fn query_redis_pool(
        &mut self,
        sql: &str,
        ttl_minutes: i64,
    ) -> Result<Vec<HashMap<String, Value>>> {
        // Try to get from Redis pool first
        if let Ok(cached) = self.get_from_redis_pool(sql).await {
            if let Some(data) = cached {
                return Ok(serde_json::from_str(&data)?);
            }
        }

        self.log_sql(sql, "query_redis_pool");

        // Not in cache, get from Postgres pool
        let rows = PostgresPoolClient::query(sql).await?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        // Save to Redis cache
        let _ = self.save_to_redis_pool(sql, &rows, ttl_minutes).await;

        Ok(rows)
    }

    /// Execute INSERT/UPDATE/DELETE using connection pool
    /// For HTTP API use
    pub async fn command_pool(&mut self, sql: &str) -> Result<()> {
        self.log_sql(sql, "command_pool");

        let result = PostgresPoolClient::command(sql).await?;

        self.last_id = result.last_id;
        self.affected_rows = result.affected_rows;

        Ok(())
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Convert sqlx rows to Vec<HashMap<String, Value>>
    fn rows_to_hashmap(rows: Vec<sqlx::postgres::PgRow>) -> Vec<HashMap<String, Value>> {
        let mut result = Vec::new();

        for row in rows {
            let mut map = HashMap::new();

            for (i, column) in row.columns().iter().enumerate() {
                let name = column.name().to_string();
                let value: Value = row
                    .try_get_raw(i)
                    .ok()
                    .and_then(|_raw| {
                        // Try different types
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

        result
    }

    /// Get escaped SQL string (prevents SQL injection)
    pub fn get_escaped_sql_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\").replace('\'', "\\'")
    }

    /// Get integers formatted for SQL IN clause
    /// Example: vec![1, 2, 3] -> "1, 2, 3"
    pub fn get_integers_sql_in(&self, ids: &[i64]) -> String {
        if ids.is_empty() {
            return String::new();
        }

        let mut unique_ids: Vec<i64> = ids.to_vec();
        unique_ids.sort_unstable();
        unique_ids.dedup();

        unique_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get strings formatted for SQL IN clause with quotes
    /// Example: vec!["a", "b"] -> "'a', 'b'"
    pub fn get_strings_sql_in(&self, values: &[String]) -> String {
        if values.is_empty() {
            return String::new();
        }

        let mut unique_values: Vec<String> = values
            .iter()
            .map(|v| self.get_escaped_sql_string(v))
            .collect();
        unique_values.sort();
        unique_values.dedup();

        unique_values
            .iter()
            .map(|v| format!("'{}'", v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn get_last_id(&self) -> Option<i64> {
        self.last_id
    }

    pub fn get_affected_rows(&self) -> u64 {
        self.affected_rows
    }

    // ========================================================================
    // REDIS CACHE METHODS (private)
    // ========================================================================

    async fn get_from_redis(&self, sql: &str) -> Result<Option<String>> {
        let table_name = self.get_table_name_from_sql(sql);
        let redis_key = format!(
            "{}:sql:{}:{}",
            self.environment,
            table_name,
            self.get_md5_hash(sql)
        );

        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        let result: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;

        Ok(result)
    }

    async fn save_to_redis(
        &self,
        sql: &str,
        rows: &[HashMap<String, Value>],
        ttl_minutes: i64,
    ) -> Result<()> {
        let table_name = self.get_table_name_from_sql(sql);
        let redis_key = format!(
            "{}:sql:{}:{}",
            self.environment,
            table_name,
            self.get_md5_hash(sql)
        );

        let json_data = serde_json::to_string(rows)?;

        let redis = RedisClient::instance();
        let mut conn = redis.get_connection().await?;

        redis::cmd("SET")
            .arg(&redis_key)
            .arg(&json_data)
            .arg("EX")
            .arg(ttl_minutes * 60)
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn get_from_redis_pool(&self, sql: &str) -> Result<Option<String>> {
        let table_name = self.get_table_name_from_sql(sql);
        let redis_key = format!(
            "{}:sql:{}:{}",
            self.environment,
            table_name,
            self.get_md5_hash(sql)
        );

        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        let result: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;

        RedisPoolClient::release(idx).await?;

        Ok(result)
    }

    async fn save_to_redis_pool(
        &self,
        sql: &str,
        rows: &[HashMap<String, Value>],
        ttl_minutes: i64,
    ) -> Result<()> {
        let table_name = self.get_table_name_from_sql(sql);
        let redis_key = format!(
            "{}:sql:{}:{}",
            self.environment,
            table_name,
            self.get_md5_hash(sql)
        );

        let json_data = serde_json::to_string(rows)?;

        let (idx, mut conn) = RedisPoolClient::acquire().await?;

        redis::cmd("SET")
            .arg(&redis_key)
            .arg(&json_data)
            .arg("EX")
            .arg(ttl_minutes * 60)
            .query_async::<_, ()>(&mut conn)
            .await?;

        RedisPoolClient::release(idx).await?;

        Ok(())
    }

    fn get_table_name_from_sql(&self, sql: &str) -> String {
        // Extract table name from "FROM table_name"
        let re = regex::Regex::new(r"from\s+([a-zA-Z0-9_\.]+)")
            .unwrap();

        re.captures(sql)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn get_md5_hash(&self, text: &str) -> String {
        let digest = md5::compute(text.as_bytes());
        format!("{:x}", digest)
    }

    fn log_sql(&self, sql: &str, title: &str) {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        if env != "production" {
            log_debug!("[{}] {}", title, sql);
        }

        // También log a archivo si hay un logger configurado
        // Logger::instance().log_sql(sql, title);
    }
}

impl Default for AbstractPostgresRepository {
    fn default() -> Self {
        Self::new()
    }
}
