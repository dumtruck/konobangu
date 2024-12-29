use bytes::Bytes;
use reqwest::IntoUrl;

use super::{core::DEFAULT_HTTP_CLIENT_USER_AGENT, HttpClient};

pub async fn download_bytes<T: IntoUrl>(url: T) -> eyre::Result<Bytes> {
    let request_client = reqwest::Client::builder()
        .user_agent(DEFAULT_HTTP_CLIENT_USER_AGENT)
        .build()?;
    let bytes = request_client.get(url).send().await?.bytes().await?;
    Ok(bytes)
}

pub async fn download_bytes_with_client<T: IntoUrl>(
    client: Option<&HttpClient>,
    url: T,
) -> eyre::Result<Bytes> {
    if let Some(client) = client {
        let bytes = client.get(url).send().await?.bytes().await?;
        Ok(bytes)
    } else {
        download_bytes(url).await
    }
}
