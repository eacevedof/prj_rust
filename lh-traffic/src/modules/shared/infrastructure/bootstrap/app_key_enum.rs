use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppKeyEnum {
    PostgresClient,
    RedisClient,
    Logger,
    HttpServer,
    // Add more keys as needed
}

impl fmt::Display for AppKeyEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppKeyEnum::PostgresClient => write!(f, "postgres_client"),
            AppKeyEnum::RedisClient => write!(f, "redis_client"),
            AppKeyEnum::Logger => write!(f, "logger"),
            AppKeyEnum::HttpServer => write!(f, "http_server"),
        }
    }
}
