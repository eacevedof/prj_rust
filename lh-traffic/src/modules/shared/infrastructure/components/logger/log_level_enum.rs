use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevelEnum {
    Debug,
    Info,
    Warning,
    Error,
    Security,
    Sql,
}

impl fmt::Display for LogLevelEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevelEnum::Debug => write!(f, "debug"),
            LogLevelEnum::Info => write!(f, "info"),
            LogLevelEnum::Warning => write!(f, "warning"),
            LogLevelEnum::Error => write!(f, "error"),
            LogLevelEnum::Security => write!(f, "security"),
            LogLevelEnum::Sql => write!(f, "sql"),
        }
    }
}
