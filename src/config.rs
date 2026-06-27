use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionMode {
    Unrestricted,
    Readonly,
    Restricted,
}

impl<'de> Deserialize<'de> for PermissionMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "unrestricted" => Ok(PermissionMode::Unrestricted),
            "readonly" | "read_only" => Ok(PermissionMode::Readonly),
            "restricted" => Ok(PermissionMode::Restricted),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["unrestricted", "readonly", "restricted"],
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_schema")]
    pub default_schema: String,
    #[serde(default = "default_permission_mode")]
    pub permission_mode: PermissionMode,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_max_result_rows")]
    pub max_result_rows: u32,
}

fn default_max_result_rows() -> u32 {
    1000
}

fn default_schema() -> String {
    "public".to_string()
}

fn default_permission_mode() -> PermissionMode {
    PermissionMode::Restricted
}

fn default_max_connections() -> u32 {
    5
}

pub fn load_config() -> Result<Config, String> {
    Figment::new()
        .merge(Env::raw())
        .extract()
        .map_err(|e| format!("Failed to load config: {}", e))
}
