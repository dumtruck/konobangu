use std::ops::Deref;

use loco_rs::app::{AppContext, Initializer};
use once_cell::sync::OnceCell;

use super::{AppMikanConfig, MIKAN_BASE_URL};
use crate::{config::AppConfigExt, fetch::HttpClient};

static APP_MIKAN_CLIENT: OnceCell<AppMikanClient> = OnceCell::new();

pub struct AppMikanClient {
    http_client: HttpClient,
    base_url: String,
}

impl AppMikanClient {
    pub fn new(mut config: AppMikanConfig) -> loco_rs::Result<Self> {
        let http_client =
            HttpClient::new(config.http_client.take()).map_err(loco_rs::Error::wrap)?;
        let base_url = config
            .base_url
            .unwrap_or_else(|| String::from(MIKAN_BASE_URL));
        Ok(Self {
            http_client,
            base_url,
        })
    }

    pub fn global() -> &'static AppMikanClient {
        APP_MIKAN_CLIENT
            .get()
            .expect("Global mikan http client is not initialized")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Deref for AppMikanClient {
    type Target = HttpClient;

    fn deref(&self) -> &Self::Target {
        &self.http_client
    }
}

pub struct AppMikanClientInitializer;

#[async_trait::async_trait]
impl Initializer for AppMikanClientInitializer {
    fn name(&self) -> String {
        "AppMikanClientInitializer".to_string()
    }

    async fn before_run(&self, app_context: &AppContext) -> loco_rs::Result<()> {
        let config = &app_context.config;
        let app_mikan_conf = config.get_mikan_conf()?.unwrap_or_default();

        APP_MIKAN_CLIENT.get_or_try_init(|| AppMikanClient::new(app_mikan_conf))?;

        Ok(())
    }
}
