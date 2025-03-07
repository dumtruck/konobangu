use color_eyre::eyre;
use reqwest::IntoUrl;

use crate::{
    extract::mikan::{AppMikanClient, AppMikanConfig},
    fetch::HttpClientConfig,
};

pub fn build_testing_mikan_client(base_mikan_url: impl IntoUrl) -> eyre::Result<AppMikanClient> {
    let mikan_client = AppMikanClient::new(AppMikanConfig {
        http_client: HttpClientConfig {
            ..Default::default()
        },
        base_url: base_mikan_url.into_url()?,
    })?;
    Ok(mikan_client)
}
