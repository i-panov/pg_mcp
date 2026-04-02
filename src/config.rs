use figment::{providers::Env, Figment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum PermissionMode {
    Unrestricted,
    Readonly,
    Restricted,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_schema")]
    pub default_schema: String,
    #[serde(default = "default_permission_mode")]
    #[allow(dead_code)]
    pub permission_mode: PermissionMode,
}

fn default_schema() -> String {
    "public".to_string()
}

fn default_permission_mode() -> PermissionMode {
    PermissionMode::Restricted
}

pub fn load_config() -> Config {
    Figment::new()
        .merge(Env::raw())
        .extract()
        .expect("Failed to load config: DATABASE_URL must be set")
}
