use fetch::{FetchError, HttpClientConfig, IntoUrl};

use crate::{
    errors::RecorderResult,
    extract::mikan::{MikanClient, MikanConfig},
};

pub async fn build_testing_mikan_client(
    base_mikan_url: impl IntoUrl,
) -> RecorderResult<MikanClient> {
    let mikan_client = MikanClient::from_config(MikanConfig {
        http_client: HttpClientConfig {
            ..Default::default()
        },
        base_url: base_mikan_url.into_url().map_err(FetchError::from)?,
    })
    .await?;
    Ok(mikan_client)
}
