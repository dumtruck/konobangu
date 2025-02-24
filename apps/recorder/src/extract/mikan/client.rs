use std::ops::Deref;

use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use once_cell::sync::OnceCell;
use url::Url;

use super::AppMikanConfig;
use crate::{config::AppConfigExt, fetch::HttpClient};

static APP_MIKAN_CLIENT: OnceCell<AppMikanClient> = OnceCell::new();

#[derive(Debug)]
pub struct AppMikanClient {
    http_client: HttpClient,
    base_url: Url,
}

impl AppMikanClient {
    pub fn new(config: AppMikanConfig) -> loco_rs::Result<Self> {
        let http_client =
            HttpClient::from_config(config.http_client).map_err(loco_rs::Error::wrap)?;
        let base_url = config.base_url;
        Ok(Self {
            http_client,
            base_url,
        })
    }

    pub fn app_instance() -> &'static AppMikanClient {
        APP_MIKAN_CLIENT
            .get()
            .expect("AppMikanClient is not initialized")
    }

    pub fn base_url(&self) -> &Url {
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

#[async_trait]
impl Initializer for AppMikanClientInitializer {
    fn name(&self) -> String {
        "AppMikanClientInitializer".to_string()
    }

    async fn before_run(&self, app_context: &AppContext) -> loco_rs::Result<()> {
        let config = &app_context.config;
        let app_mikan_conf = config.get_app_conf()?.mikan;

        APP_MIKAN_CLIENT.get_or_try_init(|| AppMikanClient::new(app_mikan_conf))?;

        Ok(())
    }
}
