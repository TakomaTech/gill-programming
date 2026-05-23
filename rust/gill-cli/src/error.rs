use thiserror::Error;

#[derive(Debug, Error)]
pub enum GillError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Invalid expression")]
    InvalidExpression,
    #[error("Runtime failure: {0}")]
    RuntimeError(String),
}
