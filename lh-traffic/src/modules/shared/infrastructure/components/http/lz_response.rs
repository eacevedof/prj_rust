use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// HTTP Response wrapper
/// Equivalent to TypeScript's LzResponse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LzResponse {
    #[serde(skip)]
    pub headers: HashMap<String, String>,
    pub status_code: u16,
    #[serde(skip)]
    pub media_type: String,
    #[serde(skip)]
    pub is_body_writable: bool,
    pub body: Value,
}

impl LzResponse {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            status_code: 200,
            media_type: String::from("application/json"),
            is_body_writable: true,
            body: Value::Null,
        }
    }

    pub fn with_status(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn with_body(mut self, body: Value) -> Self {
        self.body = body;
        self
    }

    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn set_media_type(mut self, media_type: String) -> Self {
        self.media_type = media_type;
        self
    }

    /// Create success response
    pub fn success(data: Value) -> Self {
        Self::new()
            .with_status(200)
            .with_body(serde_json::json!({
                "code": 200,
                "message": "success",
                "data": data
            }))
    }

    /// Create error response
    pub fn error(message: &str, code: u16) -> Self {
        Self::new()
            .with_status(code)
            .with_body(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
    }
}

impl Default for LzResponse {
    fn default() -> Self {
        Self::new()
    }
}
