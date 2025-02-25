use std::ops::Deref;

use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use once_cell::sync::OnceCell;
use reqwest_middleware::ClientWithMiddleware;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use super::AppMikanConfig;
use crate::{
    config::AppConfigExt,
    errors::RecorderError,
    fetch::{HttpClient, HttpClientTrait, client::HttpClientCookiesAuth},
};

static APP_MIKAN_CLIENT: OnceCell<AppMikanClient> = OnceCell::new();

#[derive(Debug, Default, Clone)]
pub struct MikanAuthSecrecy {
    pub cookie: SecretString,
    pub user_agent: Option<String>,
}

impl MikanAuthSecrecy {
    pub fn into_cookie_auth(self, url: &Url) -> Result<HttpClientCookiesAuth, RecorderError> {
        HttpClientCookiesAuth::from_cookies(self.cookie.expose_secret(), url, self.user_agent)
    }
}

#[derive(Debug)]
pub struct AppMikanClient {
    http_client: HttpClient,
    base_url: Url,
}

impl AppMikanClient {
    pub fn new(config: AppMikanConfig) -> Result<Self, RecorderError> {
        let http_client = HttpClient::from_config(config.http_client)?;
        let base_url = config.base_url;
        Ok(Self {
            http_client,
            base_url,
        })
    }

    pub fn fork_with_auth(&self, secrecy: MikanAuthSecrecy) -> Result<Self, RecorderError> {
        let cookie_auth = secrecy.into_cookie_auth(&self.base_url)?;
        let fork = self.http_client.fork().attach_secrecy(cookie_auth);

        Ok(Self {
            http_client: HttpClient::from_fork(fork)?,
            base_url: self.base_url.clone(),
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

    pub fn client(&self) -> &HttpClient {
        &self.http_client
    }
}

impl Deref for AppMikanClient {
    type Target = ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        self.http_client.deref()
    }
}

impl HttpClientTrait for AppMikanClient {}

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
