use bytes::Bytes;
use reqwest::IntoUrl;

use super::defs::DEFAULT_USER_AGENT;

pub async fn download_bytes<T: IntoUrl>(url: T) -> eyre::Result<Bytes> {
    let request_client = reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build()?;
    let bytes = request_client.get(url).send().await?.bytes().await?;
    Ok(bytes)
}
