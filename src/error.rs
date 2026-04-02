use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database connection failed: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Query execution failed: {0}")]
    QueryExecution(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
