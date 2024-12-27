use serde::{Deserialize, Serialize};

use crate::fetch::HttpClientConfig;

pub const MIKAN_CONF_KEY: &str = "mikan";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppMikanConfig {
    pub http_client: Option<HttpClientConfig>,
    pub base_url: Option<String>,
}
