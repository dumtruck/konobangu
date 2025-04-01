use std::{fmt::Debug, ops::Deref, sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::http::{self, Extensions};
use http_cache_reqwest::{
    Cache, CacheManager, CacheMode, HttpCache, HttpCacheOptions, MokaManager,
};
use leaky_bucket::RateLimiter;
use reqwest::{ClientBuilder, Request, Response};
use reqwest_middleware::{
    ClientBuilder as ClientWithMiddlewareBuilder, ClientWithMiddleware, Middleware, Next,
};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use snafu::Snafu;

use super::HttpClientSecrecyDataTrait;
use crate::fetch::get_random_mobile_ua;

pub struct RateLimiterMiddleware {
    rate_limiter: RateLimiter,
}

#[async_trait]
impl Middleware for RateLimiterMiddleware {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HttpClientCacheBackendConfig {
    Moka { cache_size: u64 },
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

pub(crate) struct CacheBackend(Box<dyn CacheManager>);

impl CacheBackend {
    pub(crate) fn new<T: CacheManager>(backend: T) -> Self {
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

#[derive(Debug, Snafu)]
pub enum HttpClientError {
    #[snafu(transparent)]
    ReqwestError { source: reqwest::Error },
    #[snafu(transparent)]
    ReqwestMiddlewareError { source: reqwest_middleware::Error },
    #[snafu(transparent)]
    HttpError { source: http::Error },
}

pub trait HttpClientTrait: Deref<Target = ClientWithMiddleware> + Debug {}

pub struct HttpClientFork {
    pub client_builder: ClientBuilder,
    pub middleware_stack: Vec<Arc<dyn Middleware>>,
    pub config: HttpClientConfig,
}

impl HttpClientFork {
    pub fn attach_secrecy<S: HttpClientSecrecyDataTrait>(self, secrecy: S) -> Self {
        let mut fork = self;
        fork.client_builder = secrecy.attach_secrecy_to_client(fork.client_builder);
        fork
    }
}

pub struct HttpClient {
    client: ClientWithMiddleware,
    middleware_stack: Vec<Arc<dyn Middleware>>,
    pub config: HttpClientConfig,
}

impl Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("config", &self.config)
            .finish()
    }
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

impl HttpClient {
    pub fn from_config(config: HttpClientConfig) -> Result<Self, HttpClientError> {
        let mut middleware_stack: Vec<Arc<dyn Middleware>> = vec![];
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
        let mut reqwest_with_middleware_builder = ClientWithMiddlewareBuilder::new(reqwest_client);

        {
            let tracing_middleware = Arc::new(TracingMiddleware::default());

            middleware_stack.push(tracing_middleware.clone());

            reqwest_with_middleware_builder =
                reqwest_with_middleware_builder.with_arc(tracing_middleware)
        }

        {
            if let Some(ref x) = config.exponential_backoff_max_retries {
                let retry_policy = ExponentialBackoff::builder().build_with_max_retries(*x);

                let retry_transient_middleware =
                    Arc::new(RetryTransientMiddleware::new_with_policy(retry_policy));

                middleware_stack.push(retry_transient_middleware.clone());

                reqwest_with_middleware_builder =
                    reqwest_with_middleware_builder.with_arc(retry_transient_middleware);
            }
        }

        {
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

                let rate_limiter_middleware = Arc::new(RateLimiterMiddleware { rate_limiter });

                middleware_stack.push(rate_limiter_middleware.clone());

                reqwest_with_middleware_builder =
                    reqwest_with_middleware_builder.with_arc(rate_limiter_middleware);
            }
        }

        {
            if let (None, None) = (config.cache_backend.as_ref(), config.cache_preset.as_ref()) {
            } else {
                let cache_preset = config.cache_preset.as_ref().cloned().unwrap_or_default();
                let cache_backend = config
                    .cache_backend
                    .as_ref()
                    .map(|b| match b {
                        HttpClientCacheBackendConfig::Moka { cache_size } => {
                            CacheBackend::new(MokaManager {
                                cache: Arc::new(moka::future::Cache::new(*cache_size)),
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

                let http_cache_middleware = Arc::new(Cache(http_cache));

                middleware_stack.push(http_cache_middleware.clone());

                reqwest_with_middleware_builder =
                    reqwest_with_middleware_builder.with_arc(http_cache_middleware);
            }
        }

        let reqwest_with_middleware = reqwest_with_middleware_builder.build();

        Ok(Self {
            client: reqwest_with_middleware,
            middleware_stack,
            config,
        })
    }

    pub fn fork(&self) -> HttpClientFork {
        let reqwest_client_builder = ClientBuilder::new().user_agent(
            self.config
                .user_agent
                .as_deref()
                .unwrap_or_else(|| get_random_mobile_ua()),
        );

        #[cfg(not(target_arch = "wasm32"))]
        let reqwest_client_builder =
            reqwest_client_builder.redirect(reqwest::redirect::Policy::none());

        HttpClientFork {
            client_builder: reqwest_client_builder,
            middleware_stack: self.middleware_stack.clone(),
            config: self.config.clone(),
        }
    }

    pub fn from_fork(fork: HttpClientFork) -> Result<Self, HttpClientError> {
        let HttpClientFork {
            client_builder,
            middleware_stack,
            config,
        } = fork;
        let reqwest_client = client_builder.build()?;
        let mut reqwest_with_middleware_builder = ClientWithMiddlewareBuilder::new(reqwest_client);

        for m in &middleware_stack {
            reqwest_with_middleware_builder = reqwest_with_middleware_builder.with_arc(m.clone());
        }

        let reqwest_with_middleware = reqwest_with_middleware_builder.build();

        Ok(Self {
            client: reqwest_with_middleware,
            middleware_stack,
            config,
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient::from_config(Default::default()).expect("Failed to create default HttpClient")
    }
}

impl HttpClientTrait for HttpClient {}
