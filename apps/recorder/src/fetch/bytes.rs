use bytes::Bytes;
use reqwest::IntoUrl;

use super::HttpClient;

pub async fn fetch_bytes<T: IntoUrl>(client: Option<&HttpClient>, url: T) -> color_eyre::eyre::Result<Bytes> {
    let client = client.unwrap_or_default();

    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes)
}
