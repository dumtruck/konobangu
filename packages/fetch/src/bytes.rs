use bytes::Bytes;
use reqwest::IntoUrl;

use super::client::HttpClientTrait;
use crate::FetchError;

pub async fn fetch_bytes<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> Result<Bytes, FetchError> {
    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes)
}
