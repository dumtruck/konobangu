use std::sync::{Arc, RwLock, Weak};

use lazy_static::lazy_static;
use opendal::raw::Accessor;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use tokio_utils::RateLimiter;
use weak_table::WeakValueHashMap;

use crate::downloaders::defs::DEFAULT_USER_AGENT;

pub(crate) const TMDB_API_ORIGIN: &str = "https://api.themoviedb.org";

pub struct TmdbApiClient {
    api_token: String,
    rate_limiter: RateLimiter,
    fetch_client: reqwest::Client,
    headers: HeaderMap,
}

lazy_static! {
    static ref TMDB_API_CLIENT_MAP: Arc<RwLock<WeakValueHashMap<String, Weak<TmdbApiClient>>>> =
        { Arc::new(RwLock::new(WeakValueHashMap::new())) };
}

impl TmdbApiClient {
    pub async fn new<S: AsRef<str>>(api_token: S) -> Arc<Self> {
        let api_token = api_token.as_ref();
        let map_read = TMDB_API_CLIENT_MAP.read().await;
        if let Some(client) = map_read.get(api_token) {
            return client.clone();
        }
        let client = Arc::new(TmdbApiClient {
            api_token: api_token.to_string(),
            rate_limiter: RateLimiter::new(std::time::Duration::from_millis(50)),
            fetch_client: reqwest::Client::builder()
                .user_agent(DEFAULT_USER_AGENT)
                .build(),
            headers: {
                let mut header_map = HeaderMap::new();
                header_map.insert(ACCEPT, HeaderValue::from("application/json"));
                header_map.insert(
                    AUTHORIZATION,
                    HeaderValue::from(format!("Bearer {api_token}")),
                );
                header_map
            },
        });
        {
            let mut map_write = TMDB_API_CLIENT_MAP.write().await;
            map_write.insert(api_token.to_string(), client.clone());
        }
        client.clone()
    }

    pub fn get_api_token(&self) -> &str {
        &self.api_token
    }

    pub async fn fetch<R, F>(&self, f: F) -> Result<R, reqwest::Error>
    where
        F: FnOnce(&reqwest::Client) -> reqwest::RequestBuilder,
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
}
