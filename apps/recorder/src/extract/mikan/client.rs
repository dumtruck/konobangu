use std::ops::Deref;

use reqwest_middleware::ClientWithMiddleware;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use super::MikanConfig;
use crate::{
    errors::RError,
    fetch::{HttpClient, HttpClientTrait, client::HttpClientCookiesAuth},
};

#[derive(Debug, Default, Clone)]
pub struct MikanAuthSecrecy {
    pub cookie: SecretString,
    pub user_agent: Option<String>,
}

impl MikanAuthSecrecy {
    pub fn into_cookie_auth(self, url: &Url) -> Result<HttpClientCookiesAuth, RError> {
        HttpClientCookiesAuth::from_cookies(self.cookie.expose_secret(), url, self.user_agent)
    }
}

#[derive(Debug)]
pub struct MikanClient {
    http_client: HttpClient,
    base_url: Url,
}

impl MikanClient {
    pub async fn from_config(config: MikanConfig) -> Result<Self, RError> {
        let http_client = HttpClient::from_config(config.http_client)?;
        let base_url = config.base_url;
        Ok(Self {
            http_client,
            base_url,
        })
    }

    pub fn fork_with_auth(&self, secrecy: MikanAuthSecrecy) -> Result<Self, RError> {
        let cookie_auth = secrecy.into_cookie_auth(&self.base_url)?;
        let fork = self.http_client.fork().attach_secrecy(cookie_auth);

        Ok(Self {
            http_client: HttpClient::from_fork(fork)?,
            base_url: self.base_url.clone(),
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn client(&self) -> &HttpClient {
        &self.http_client
    }
}

impl Deref for MikanClient {
    type Target = ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        self.http_client.deref()
    }
}

impl HttpClientTrait for MikanClient {}
