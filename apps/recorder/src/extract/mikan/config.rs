use serde::{Deserialize, Serialize};

use crate::fetch::HttpClientConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppMikanConfig {
    pub http_client: Option<HttpClientConfig>,
    pub base_url: Option<String>,
}
