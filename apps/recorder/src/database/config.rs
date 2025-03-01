use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub enable_logging: bool,
    pub min_connections: u32,
    pub max_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub acquire_timeout: Option<u64>,
    #[serde(default)]
    pub auto_migrate: bool,
}
