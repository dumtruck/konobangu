use serde::{Deserialize, Serialize};
use url::Url;

use crate::fetch::HttpClientConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppMikanConfig {
    pub http_client: HttpClientConfig,
    pub base_url: Url,
}
