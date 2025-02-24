use bytes::Bytes;
use reqwest::IntoUrl;

use super::client::HttpClientTrait;

pub async fn fetch_bytes<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> color_eyre::eyre::Result<Bytes> {
    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes)
}
