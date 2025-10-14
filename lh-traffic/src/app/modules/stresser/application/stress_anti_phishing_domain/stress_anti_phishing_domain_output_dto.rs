use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressAntiPhishingDomainOutputDto {
    /// Total requests sent
    pub total_requests: u64,

    /// Successful responses (2xx)
    pub successful_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// Min response time in milliseconds
    pub min_response_time_ms: f64,

    /// Max response time in milliseconds
    pub max_response_time_ms: f64,

    /// Requests per second achieved
    pub actual_rps: f64,

    /// Total duration in seconds
    pub total_duration_seconds: f64,

    /// HTTP status code breakdown
    pub status_codes: std::collections::HashMap<u16, u64>,
}

impl StressAntiPhishingDomainOutputDto {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: f64::MAX,
            max_response_time_ms: 0.0,
            actual_rps: 0.0,
            total_duration_seconds: 0.0,
            status_codes: std::collections::HashMap::new(),
        }
    }
}

impl Default for StressAntiPhishingDomainOutputDto {
    fn default() -> Self {
        Self::new()
    }
}
