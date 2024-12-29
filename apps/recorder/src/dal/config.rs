use serde::{Deserialize, Serialize};

pub const DAL_CONF_KEY: &str = "dal";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppDalConfig {
    pub data_dir: Option<String>,
}
