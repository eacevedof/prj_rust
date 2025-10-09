use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentEnum {
    Development,
    Production,
    Testing,
}

impl EnvironmentEnum {
    pub fn from_env() -> Self {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        match env.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Development,
        }
    }
}

impl fmt::Display for EnvironmentEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentEnum::Development => write!(f, "development"),
            EnvironmentEnum::Production => write!(f, "production"),
            EnvironmentEnum::Testing => write!(f, "testing"),
        }
    }
}
