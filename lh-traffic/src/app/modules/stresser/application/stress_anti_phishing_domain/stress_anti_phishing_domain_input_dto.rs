use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressAntiPhishingDomainInputDto {
    /// API endpoint URL
    pub api_url: String,

    /// Device auth token
    pub device_auth_token: String,

    /// Requests per second
    pub requests_per_second: u64,

    /// Total duration in seconds
    pub duration_seconds: u64,

    /// Optional: custom list of domains to test (if empty, uses default list)
    pub custom_domains: Vec<String>,
}

impl StressAntiPhishingDomainInputDto {
    pub fn new(
        api_url: String,
        device_auth_token: String,
        requests_per_second: u64,
        duration_seconds: u64,
    ) -> Self {
        Self {
            api_url,
            device_auth_token,
            requests_per_second,
            duration_seconds,
            custom_domains: Vec::new(),
        }
    }

    pub fn with_custom_domains(mut self, domains: Vec<String>) -> Self {
        self.custom_domains = domains;
        self
    }
}
