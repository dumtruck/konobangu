use std::{
    ops::Deref,
    sync::{Arc, Weak},
};

use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use tokio::sync::RwLock;
use weak_table::WeakValueHashMap;

use crate::downloaders::ApiClient;

pub(crate) const TMDB_API_ORIGIN: &str = "https://api.themoviedb.org";

pub struct TmdbApiClient {
    api_token: String,
    api_client: ApiClient,
}

lazy_static! {
    static ref TMDB_API_CLIENT_MAP: Arc<RwLock<WeakValueHashMap<String, Weak<TmdbApiClient>>>> =
        Arc::new(RwLock::new(WeakValueHashMap::new()));
}

impl TmdbApiClient {
    pub async fn new<S: AsRef<str>>(api_token: S) -> eyre::Result<Arc<Self>> {
        let api_token = api_token.as_ref();
        {
            let map_read = TMDB_API_CLIENT_MAP.read().await;
            if let Some(client) = map_read.get(api_token) {
                return Ok(client.clone());
            }
        }
        let client = Arc::new(TmdbApiClient {
            api_token: api_token.to_string(),
            api_client: ApiClient::new(
                std::time::Duration::from_millis(50),
                Some({
                    let mut header_map = HeaderMap::new();
                    header_map.insert(ACCEPT, HeaderValue::from_static("application/json"));
                    header_map.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&format!("Bearer {api_token}"))?,
                    );
                    header_map
                }),
            )?,
        });
        {
            let mut map_write = TMDB_API_CLIENT_MAP.write().await;
            map_write.insert(api_token.to_string(), client.clone());
        }
        Ok(client)
    }

    pub fn get_api_token(&self) -> &str {
        &self.api_token
    }
}

impl Deref for TmdbApiClient {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{env, sync::Arc};

    use crate::parsers::tmdb::tmdb_client::TmdbApiClient;

    pub async fn prepare_tmdb_api_client() -> Arc<TmdbApiClient> {
        dotenv::from_filename("test.env").expect("failed to load test.env");
        let tmdb_api_token = env::var("TMDB_API_TOKEN").expect("TMDB_API_TOKEN is not set");
        TmdbApiClient::new(tmdb_api_token)
            .await
            .expect("failed to create tmdb api client")
    }
}
