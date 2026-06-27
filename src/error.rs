use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Permission denied: {0}")]
    Permission(String),
}
