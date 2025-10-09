use thiserror::Error;

#[derive(Error, Debug)]
pub enum LzRequestException {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Missing header: {0}")]
    MissingHeader(String),

    #[error("Invalid body: {0}")]
    InvalidBody(String),
}
