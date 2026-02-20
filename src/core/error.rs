use thiserror::Error;

#[derive(Debug, Error)]
pub enum HitLeidenError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("backend error: {0}")]
    Backend(String),
    #[error("acceleration error: {0}")]
    Acceleration(String),
}
