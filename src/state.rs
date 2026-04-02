use crate::config::Config;
use sqlx::postgres::PgPool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub default_schema: String,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(&config.database_url).await?;
        Ok(Self {
            pool,
            default_schema: config.default_schema,
        })
    }
}
