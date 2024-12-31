use std::{ops::Deref, time::Duration};

use axum::http::Extensions;
use leaky_bucket::RateLimiter;
use once_cell::sync::OnceCell;
use reqwest::{ClientBuilder, Request, Response};
use reqwest_middleware::{
    ClientBuilder as ClientWithMiddlewareBuilder, ClientWithMiddleware, Next,
};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::fetch::DEFAULT_HTTP_CLIENT_USER_AGENT;

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
    pub config: HttpClientConfig,
}

impl Into<ClientWithMiddleware> for HttpClient {
    fn into(self) -> ClientWithMiddleware {
        self.client
    }
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
    pub fn from_config(config: HttpClientConfig) -> reqwest::Result<Self> {
        let reqwest_client_builder = ClientBuilder::new().user_agent(
            config
                .user_agent
                .as_deref()
                .unwrap_or(DEFAULT_HTTP_CLIENT_USER_AGENT),
        );

        let reqwest_client = reqwest_client_builder.build()?;

        let mut reqwest_with_middleware_builder =
            ClientWithMiddlewareBuilder::new(reqwest_client).with(TracingMiddleware::default());

        if let Some(ref x) = config.exponential_backoff_max_retries {
            let retry_policy = ExponentialBackoff::builder().build_with_max_retries(*x);

            reqwest_with_middleware_builder = reqwest_with_middleware_builder
                .with(RetryTransientMiddleware::new_with_policy(retry_policy));
        }

        if let (None, None, None, None) = (
            config.leaky_bucket_initial_tokens.as_ref(),
            config.leaky_bucket_refill_tokens.as_ref(),
            config.leaky_bucket_refill_interval.as_ref(),
            config.leaky_bucket_max_tokens.as_ref(),
        ) {
        } else {
            let mut rate_limiter_builder = RateLimiter::builder();

            if let Some(ref x) = config.leaky_bucket_max_tokens {
                rate_limiter_builder.max(*x as usize);
            }
            if let Some(ref x) = config.leaky_bucket_initial_tokens {
                rate_limiter_builder.initial(*x as usize);
            }
            if let Some(ref x) = config.leaky_bucket_refill_tokens {
                rate_limiter_builder.refill(*x as usize);
            }
            if let Some(ref x) = config.leaky_bucket_refill_interval {
                rate_limiter_builder.interval(*x);
            }

            let rate_limiter = rate_limiter_builder.build();

            reqwest_with_middleware_builder =
                reqwest_with_middleware_builder.with(RateLimiterMiddleware { rate_limiter });
        }

        let reqwest_with_middleware = reqwest_with_middleware_builder.build();

        Ok(Self {
            client: reqwest_with_middleware,
            config,
        })
    }
}

static DEFAULT_HTTP_CLIENT: OnceCell<HttpClient> = OnceCell::new();

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient::from_config(Default::default()).expect("Failed to create default HttpClient")
    }
}

impl Default for &HttpClient {
    fn default() -> Self {
        DEFAULT_HTTP_CLIENT.get_or_init(HttpClient::default)
    }
}
