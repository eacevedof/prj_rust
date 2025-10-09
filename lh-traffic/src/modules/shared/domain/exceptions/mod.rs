use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainException {
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
