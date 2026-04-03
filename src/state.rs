use crate::config::{Config, PermissionMode};
use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub default_schema: String,
    pub permission_mode: PermissionMode,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .connect(&config.database_url)
            .await?;
        Ok(Self {
            pool,
            default_schema: config.default_schema,
            permission_mode: config.permission_mode,
        })
    }
}
