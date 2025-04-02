use reqwest::IntoUrl;

use crate::{
    errors::app_error::RResult,
    extract::mikan::{MikanClient, MikanConfig},
    fetch::HttpClientConfig,
};

pub async fn build_testing_mikan_client(base_mikan_url: impl IntoUrl) -> RResult<MikanClient> {
    let mikan_client = MikanClient::from_config(MikanConfig {
        http_client: HttpClientConfig {
            ..Default::default()
        },
        base_url: base_mikan_url.into_url()?,
    })
    .await?;
    Ok(mikan_client)
}
