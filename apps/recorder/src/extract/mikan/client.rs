use std::{fmt::Debug, ops::Deref};

use fetch::{HttpClient, HttpClientTrait, client::HttpClientCookiesAuth};
use serde::{Deserialize, Serialize};
use url::Url;

use super::MikanConfig;
use crate::errors::RecorderError;
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct MikanAuthSecrecy {
    pub cookie: String,
    pub user_agent: Option<String>,
}

impl Debug for MikanAuthSecrecy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MikanAuthSecrecy")
            .field("cookie", &String::from("[secrecy]"))
            .field("user_agent", &String::from("[secrecy]"))
            .finish()
    }
}

impl MikanAuthSecrecy {
    pub fn into_cookie_auth(self, url: &Url) -> Result<HttpClientCookiesAuth, RecorderError> {
        HttpClientCookiesAuth::from_cookies(&self.cookie, url, self.user_agent)
            .map_err(RecorderError::from)
    }
}

#[derive(Debug)]
pub struct MikanClient {
    http_client: HttpClient,
    base_url: Url,
}

impl MikanClient {
    pub async fn from_config(config: MikanConfig) -> Result<Self, RecorderError> {
        let http_client = HttpClient::from_config(config.http_client)?;
        let base_url = config.base_url;
        Ok(Self {
            http_client,
            base_url,
        })
    }

    pub fn fork_with_auth(&self, secrecy: Option<MikanAuthSecrecy>) -> Result<Self, RecorderError> {
        let mut fork = self.http_client.fork();

        if let Some(secrecy) = secrecy {
            let cookie_auth = secrecy.into_cookie_auth(&self.base_url)?;
            fork = fork.attach_secrecy(cookie_auth);
        }

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
    type Target = fetch::reqwest_middleware::ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        &self.http_client
    }
}

impl HttpClientTrait for MikanClient {}
