use std::sync::Arc;

use cookie::Cookie;
use reqwest::{ClientBuilder, cookie::Jar};
use url::Url;

use crate::errors::app_error::RError;

pub trait HttpClientSecrecyDataTrait {
    fn attach_secrecy_to_client(&self, client_builder: ClientBuilder) -> ClientBuilder {
        client_builder
    }
}

#[derive(Default)]
pub struct HttpClientCookiesAuth {
    pub cookie_jar: Arc<Jar>,
    pub user_agent: Option<String>,
}

impl HttpClientCookiesAuth {
    pub fn from_cookies(
        cookies: &str,
        url: &Url,
        user_agent: Option<String>,
    ) -> Result<Self, RError> {
        let cookie_jar = Arc::new(Jar::default());
        for cookie in Cookie::split_parse(cookies).try_collect::<Vec<_>>()? {
            cookie_jar.add_cookie_str(&cookie.to_string(), url);
        }

        Ok(Self {
            cookie_jar,
            user_agent,
        })
    }
}

impl HttpClientSecrecyDataTrait for HttpClientCookiesAuth {
    fn attach_secrecy_to_client(&self, client_builder: ClientBuilder) -> ClientBuilder {
        let mut client_builder = client_builder.cookie_provider(self.cookie_jar.clone());
        if let Some(ref user_agent) = self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }
        client_builder
    }
}
