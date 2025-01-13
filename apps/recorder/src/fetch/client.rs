use std::{ops::Deref, sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::http::{self, Extensions};
use http_cache_reqwest::{
    CACacheManager, Cache, CacheManager, CacheMode, HttpCache, HttpCacheOptions, MokaManager,
};
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
use thiserror::Error;

use super::get_random_mobile_ua;
use crate::app::App;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HttpClientCacheBackendConfig {
    Moka { cache_size: u64 },
    CACache { cache_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpClientCachePresetConfig {
    #[serde(rename = "rfc7234")]
    RFC7234,
}

impl Default for HttpClientCachePresetConfig {
    fn default() -> Self {
        Self::RFC7234
    }
}

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
    pub cache_backend: Option<HttpClientCacheBackendConfig>,
    pub cache_preset: Option<HttpClientCachePresetConfig>,
}

struct CacheBackend(Box<dyn CacheManager>);

impl CacheBackend {
    fn new<T: CacheManager>(backend: T) -> Self {
        Self(Box::new(backend))
    }
}

#[async_trait::async_trait]
impl CacheManager for CacheBackend {
    async fn get(
        &self,
        cache_key: &str,
    ) -> http_cache::Result<Option<(http_cache::HttpResponse, http_cache_semantics::CachePolicy)>>
    {
        self.0.get(cache_key).await
    }

    /// Attempts to cache a response and related policy.
    async fn put(
        &self,
        cache_key: String,
        res: http_cache::HttpResponse,
        policy: http_cache_semantics::CachePolicy,
    ) -> http_cache::Result<http_cache::HttpResponse> {
        self.0.put(cache_key, res, policy).await
    }
    /// Attempts to remove a record from cache.
    async fn delete(&self, cache_key: &str) -> http_cache::Result<()> {
        self.0.delete(cache_key).await
    }
}

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    HttpError(#[from] http::Error),
}

pub struct HttpClient {
    client: ClientWithMiddleware,
    pub config: HttpClientConfig,
}

impl From<HttpClient> for ClientWithMiddleware {
    fn from(val: HttpClient) -> Self {
        val.client
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

#[async_trait]
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
    pub fn from_config(config: HttpClientConfig) -> Result<Self, HttpClientError> {
        let reqwest_client_builder = ClientBuilder::new().user_agent(
            config
                .user_agent
                .as_deref()
                .unwrap_or_else(|| get_random_mobile_ua()),
        );

        #[cfg(not(target_arch = "wasm32"))]
        let reqwest_client_builder =
            reqwest_client_builder.redirect(reqwest::redirect::Policy::none());

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

        if let (None, None) = (config.cache_backend.as_ref(), config.cache_preset.as_ref()) {
        } else {
            let cache_preset = config.cache_preset.as_ref().cloned().unwrap_or_default();
            let cache_backend = config
                .cache_backend
                .as_ref()
                .map(|b| match b {
                    HttpClientCacheBackendConfig::CACache { cache_path } => {
                        let path = std::path::PathBuf::from(
                            App::get_working_root().join(cache_path).as_str(),
                        );
                        CacheBackend::new(CACacheManager { path })
                    }
                    HttpClientCacheBackendConfig::Moka { cache_size } => {
                        CacheBackend::new(MokaManager {
                            cache: Arc::new(moka::future::Cache::new(u64::max(*cache_size, 1))),
                        })
                    }
                })
                .unwrap_or_else(|| CacheBackend::new(MokaManager::default()));

            let http_cache = match cache_preset {
                HttpClientCachePresetConfig::RFC7234 => HttpCache {
                    mode: CacheMode::Default,
                    manager: cache_backend,
                    options: HttpCacheOptions::default(),
                },
            };
            reqwest_with_middleware_builder =
                reqwest_with_middleware_builder.with(Cache(http_cache));
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
