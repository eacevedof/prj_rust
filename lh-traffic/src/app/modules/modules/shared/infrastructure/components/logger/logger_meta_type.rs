#[derive(Debug, Clone)]
pub struct LoggerMetaType {
    pub request_ip: String,
    pub request_uri: String,
}

impl LoggerMetaType {
    pub fn new(request_ip: String, request_uri: String) -> Self {
        Self {
            request_ip,
            request_uri,
        }
    }
}

impl Default for LoggerMetaType {
    fn default() -> Self {
        Self {
            request_ip: String::from("unknown"),
            request_uri: String::from("unknown"),
        }
    }
}
