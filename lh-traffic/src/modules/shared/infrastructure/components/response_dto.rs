use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDto {
    pub code: u16,
    pub message: String,
    pub data: Value,
}

impl ResponseDto {
    pub fn new(code: u16, message: String, data: Value) -> Self {
        Self { code, message, data }
    }

    pub fn success(data: Value) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data,
        }
    }

    pub fn error(message: String, code: u16) -> Self {
        Self {
            code,
            message,
            data: Value::Null,
        }
    }
}
