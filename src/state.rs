use crate::config::{Config, PermissionMode};
use sqlx::postgres::PgPool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub default_schema: String,
    pub permission_mode: PermissionMode,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(&config.database_url).await?;
        Ok(Self {
            pool,
            default_schema: config.default_schema,
            permission_mode: config.permission_mode,
        })
    }
}
