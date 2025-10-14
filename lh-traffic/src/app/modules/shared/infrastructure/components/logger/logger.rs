use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Local;
use std::path::PathBuf;
use tokio::fs::{OpenOptions, create_dir_all};
use tokio::io::AsyncWriteExt;
use once_cell::sync::Lazy;

use super::{LogLevelEnum, LoggerMetaType};

/// Logger singleton
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
        let logs_path: PathBuf = std::env::current_dir()
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
        let mut meta_data: tokio::sync::RwLockWriteGuard<LoggerMetaType> = self.meta_data.write().await;
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
        let meta_data: tokio::sync::RwLockReadGuard<LoggerMetaType> = self.meta_data.read().await;
        let now: chrono::DateTime<chrono::Local> = Local::now();
        let timestamp: String = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let content: String = format!(
            "\n[{}]\nrequest_ip: {}\n{}\n{}\n",
            timestamp,
            meta_data.request_ip,
            if title.is_empty() { String::new() } else { format!("{}\n", title) },
            message
        );

        let level_prefix: String = format!("[{}]", level.to_string().to_uppercase());
        let full_content: String = format!("{} {}", level_prefix, content);

        // Log to console (simple println for CLI app)
        match level {
            LogLevelEnum::Error | LogLevelEnum::Security => {
                eprintln!("{}", full_content);
            }
            LogLevelEnum::Warning => {
                eprintln!("{}", full_content);
            }
            LogLevelEnum::Debug => {
                println!("{}", full_content);
            }
            _ => {
                println!("{}", full_content);
            }
        }

        // Log to file
        if let Err(e) = self.log_to_file(&full_content, level).await {
            eprintln!("Failed to write to log file: {}", e);
        }
    }

    /// Write log to file
    async fn log_to_file(&self, content: &str, level: LogLevelEnum) -> std::io::Result<()> {
        let today: String = Local::now().format("%Y-%m-%d").to_string();
        let extension: &str = if level == LogLevelEnum::Sql { "sql" } else { "log" };
        let filename: String = format!("{}-{}.{}", level, today, extension);

        // Ensure logs directory exists
        create_dir_all(&self.logs_path).await?;

        let file_path: PathBuf = self.logs_path.join(filename);

        let mut file: tokio::fs::File = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }
}
