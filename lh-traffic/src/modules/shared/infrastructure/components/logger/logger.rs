use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Local;
use std::path::PathBuf;
use tokio::fs::{OpenOptions, create_dir_all};
use tokio::io::AsyncWriteExt;
use once_cell::sync::Lazy;

use super::{LogLevelEnum, LoggerMetaType};

/// Logger singleton
/// Equivalent to TypeScript's Logger
pub struct Logger {
    meta_data: Arc<RwLock<LoggerMetaType>>,
    logs_path: PathBuf,
}

static INSTANCE: Lazy<Arc<Logger>> = Lazy::new(|| {
    Arc::new(Logger::new())
});

impl Logger {
    /// Get the singleton instance
    pub fn instance() -> Arc<Logger> {
        INSTANCE.clone()
    }

    /// Create a new Logger
    fn new() -> Self {
        let logs_path = std::env::current_dir()
            .unwrap_or_default()
            .join("storage")
            .join("logs");

        Self {
            meta_data: Arc::new(RwLock::new(LoggerMetaType::default())),
            logs_path,
        }
    }

    /// Set metadata for logging context
    pub async fn set_meta_data(&self, meta: LoggerMetaType) {
        let mut meta_data = self.meta_data.write().await;
        *meta_data = meta;
    }

    /// Log debug message
    pub async fn log_debug(&self, message: &str, title: &str) {
        self.log(message, title, LogLevelEnum::Debug).await;
    }

    /// Log info message
    pub async fn log_info(&self, message: &str, title: &str) {
        self.log(message, title, LogLevelEnum::Info).await;
    }

    /// Log SQL query
    pub async fn log_sql(&self, sql: &str, title: &str) {
        self.log(sql, title, LogLevelEnum::Sql).await;
    }

    /// Log error message
    pub async fn log_error(&self, message: &str, title: &str) {
        self.log(message, title, LogLevelEnum::Error).await;
    }

    /// Log security event
    pub async fn log_security(&self, message: &str, title: &str) {
        self.log(message, title, LogLevelEnum::Security).await;
    }

    /// Log warning message
    pub async fn log_warning(&self, message: &str, title: &str) {
        self.log(message, title, LogLevelEnum::Warning).await;
    }

    /// Log exception
    pub async fn log_exception(&self, error: &dyn std::error::Error, title: &str) {
        let message = format!("{:?}", error);
        self.log(&message, title, LogLevelEnum::Error).await;
    }

    /// Internal log method
    async fn log(&self, message: &str, title: &str, level: LogLevelEnum) {
        let meta_data = self.meta_data.read().await;
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let content = format!(
            "\n[{}]\nrequest_ip: {}\n{}\n{}\n",
            timestamp,
            meta_data.request_ip,
            if title.is_empty() { String::new() } else { format!("{}\n", title) },
            message
        );

        let level_prefix = format!("[{}]", level.to_string().to_uppercase());
        let full_content = format!("{} {}", level_prefix, content);

        // Log to console
        match level {
            LogLevelEnum::Error | LogLevelEnum::Security => {
                tracing::error!("{}", full_content);
            }
            LogLevelEnum::Warning => {
                tracing::warn!("{}", full_content);
            }
            LogLevelEnum::Debug => {
                tracing::debug!("{}", full_content);
            }
            _ => {
                tracing::info!("{}", full_content);
            }
        }

        // Log to file
        if let Err(e) = self.log_to_file(&full_content, level).await {
            eprintln!("Failed to write to log file: {}", e);
        }
    }

    /// Write log to file
    async fn log_to_file(&self, content: &str, level: LogLevelEnum) -> std::io::Result<()> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let extension = if level == LogLevelEnum::Sql { "sql" } else { "log" };
        let filename = format!("{}-{}.{}", level, today, extension);

        // Ensure logs directory exists
        create_dir_all(&self.logs_path).await?;

        let file_path = self.logs_path.join(filename);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }
}
