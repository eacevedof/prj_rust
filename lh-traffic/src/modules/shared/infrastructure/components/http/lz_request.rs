use std::collections::HashMap;
use serde_json::Value;

/// HTTP Request wrapper
/// Equivalent to TypeScript's LzRequest
#[derive(Debug, Clone)]
pub struct LzRequest {
    pub body: Option<Value>,
    pub has_body: bool,
    pub headers: HashMap<String, String>,
    pub remote_ip: String,
    pub mediators_ips: Vec<String>,
    pub method: String,
    pub secure: bool,
    pub url_search: HashMap<String, String>,
    pub url_params: HashMap<String, String>,
    pub pathname: String,
    pub full_url: String,
    pub user_agent: String,
}

impl LzRequest {
    pub fn new() -> Self {
        Self {
            body: None,
            has_body: false,
            headers: HashMap::new(),
            remote_ip: String::from("unknown"),
            mediators_ips: Vec::new(),
            method: String::from("GET"),
            secure: false,
            url_search: HashMap::new(),
            url_params: HashMap::new(),
            pathname: String::new(),
            full_url: String::new(),
            user_agent: String::new(),
        }
    }

    /// Get header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Get POST parameter
    pub fn get_post_parameter(&self, key: &str) -> Option<&Value> {
        self.body.as_ref()?.get(key)
    }

    /// Get URL parameter
    pub fn get_url_parameter(&self, key: &str) -> Option<&String> {
        self.url_params.get(key)
    }

    /// Get URL search parameter (query string)
    pub fn get_url_search(&self, key: &str) -> Option<&String> {
        self.url_search.get(key)
    }
}

impl Default for LzRequest {
    fn default() -> Self {
        Self::new()
    }
}
