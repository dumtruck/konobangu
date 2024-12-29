use std::{ops::Deref, time::Duration};

use axum::http::Extensions;
use leaky_bucket::RateLimiter;
use reqwest::{ClientBuilder, Request, Response};
use reqwest_middleware::{
    ClientBuilder as ClientWithMiddlewareBuilder, ClientWithMiddleware, Next,
};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::DEFAULT_HTTP_CLIENT_USER_AGENT;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HttpClientConfig {
    pub exponential_backoff_max_retries: Option<u32>,
    pub leaky_bucket_max_tokens: Option<u32>,
    pub leaky_bucket_initial_tokens: Option<u32>,
    pub leaky_bucket_refill_tokens: Option<u32>,
    #[serde_as(as = "Option<serde_with::DurationMilliSeconds>")]
    pub leaky_bucket_refill_interval: Option<Duration>,
    pub user_agent: Option<String>,
}

pub struct HttpClient {
    client: ClientWithMiddleware,
}

impl Deref for HttpClient {
    type Target = ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

pub struct RateLimiterMiddleware {
    rate_limiter: RateLimiter,
}

#[async_trait::async_trait]
impl reqwest_middleware::Middleware for RateLimiterMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &'_ mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        self.rate_limiter.acquire_one().await;
        next.run(req, extensions).await
    }
}

impl HttpClient {
    pub fn new(config: Option<HttpClientConfig>) -> reqwest::Result<Self> {
        let mut config = config.unwrap_or_default();
        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(config.exponential_backoff_max_retries.take().unwrap_or(3));
        let rate_limiter = RateLimiter::builder()
            .max(config.leaky_bucket_max_tokens.take().unwrap_or(3) as usize)
            .initial(
                config
                    .leaky_bucket_initial_tokens
                    .take()
                    .unwrap_or_default() as usize,
            )
            .refill(config.leaky_bucket_refill_tokens.take().unwrap_or(1) as usize)
            .interval(
                config
                    .leaky_bucket_refill_interval
                    .take()
                    .unwrap_or_else(|| Duration::from_millis(500)),
            )
            .build();

        let client = ClientBuilder::new()
            .user_agent(
                config
                    .user_agent
                    .take()
                    .unwrap_or_else(|| DEFAULT_HTTP_CLIENT_USER_AGENT.to_owned()),
            )
            .build()?;

        Ok(Self {
            client: ClientWithMiddlewareBuilder::new(client)
                .with(TracingMiddleware::default())
                .with(RateLimiterMiddleware { rate_limiter })
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build(),
        })
    }
}
