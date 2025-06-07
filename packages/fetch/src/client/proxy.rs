use axum::http::{HeaderMap, HeaderValue};
use reqwest::{NoProxy, Proxy};
use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};
use util::BooleanLike;

use crate::HttpClientError;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientProxyConfig {
    #[serde_as(as = "NoneAsEmptyString")]
    pub server: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub auth_header: Option<String>,
    #[serde(with = "http_serde::option::header_map")]
    pub headers: Option<HeaderMap>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub no_proxy: Option<String>,
    pub accept_invalid_certs: Option<BooleanLike>,
}

impl HttpClientProxyConfig {
    pub fn into_proxy(self) -> Result<Option<Proxy>, HttpClientError> {
        if let Some(server) = self.server {
            let mut proxy = Proxy::all(server)
                .map_err(|err| HttpClientError::ProxyParseError { source: err })?;

            if let Some(auth_header) = self.auth_header {
                let auth_header = HeaderValue::from_str(&auth_header)
                    .map_err(|_| HttpClientError::ProxyAuthHeaderParseError)?;
                proxy = proxy.custom_http_auth(auth_header);
            }

            if let Some(headers) = self.headers {
                proxy = proxy.headers(headers);
            }

            if let Some(no_proxy) = self.no_proxy {
                proxy = proxy.no_proxy(NoProxy::from_string(&no_proxy));
            }

            Ok(Some(proxy))
        } else {
            Ok(None)
        }
    }
}

impl TryFrom<HttpClientProxyConfig> for Option<Proxy> {
    type Error = HttpClientError;

    fn try_from(value: HttpClientProxyConfig) -> Result<Option<Proxy>, Self::Error> {
        value.into_proxy()
    }
}
