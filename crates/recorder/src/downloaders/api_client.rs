use axum::http::HeaderMap;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use tokio_utils::RateLimiter;

use crate::downloaders::defs::DEFAULT_USER_AGENT;

pub struct ApiClient {
    headers: HeaderMap,
    rate_limiter: RateLimiter,
    fetch_client: reqwest::Client,
}

impl ApiClient {
    pub fn new(
        throttle_duration: std::time::Duration,
        override_headers: Option<HeaderMap>,
    ) -> eyre::Result<Self> {
        Ok(Self {
            headers: override_headers.unwrap_or_else(HeaderMap::new),
            rate_limiter: RateLimiter::new(throttle_duration),
            fetch_client: reqwest::Client::builder()
                .user_agent(DEFAULT_USER_AGENT)
                .build()?,
        })
    }

    pub async fn fetch_json<R, F>(&self, f: F) -> Result<R, reqwest::Error>
    where
        F: FnOnce(&reqwest::Client) -> reqwest::RequestBuilder,
        R: DeserializeOwned,
    {
        self.rate_limiter
            .throttle(|| async {
                f(&self.fetch_client)
                    .headers(self.headers.clone())
                    .send()
                    .await?
                    .json::<R>()
                    .await
            })
            .await
    }

    pub async fn fetch_bytes<F>(&self, f: F) -> Result<Bytes, reqwest::Error>
    where
        F: FnOnce(&reqwest::Client) -> reqwest::RequestBuilder,
    {
        self.rate_limiter
            .throttle(|| async {
                f(&self.fetch_client)
                    .headers(self.headers.clone())
                    .send()
                    .await?
                    .bytes()
                    .await
            })
            .await
    }

    pub async fn fetch_text<F>(&self, f: F) -> Result<String, reqwest::Error>
    where
        F: FnOnce(&reqwest::Client) -> reqwest::RequestBuilder,
    {
        self.rate_limiter
            .throttle(|| async {
                f(&self.fetch_client)
                    .headers(self.headers.clone())
                    .send()
                    .await?
                    .text()
                    .await
            })
            .await
    }
}
