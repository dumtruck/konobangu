use bytes::Bytes;
use reqwest::IntoUrl;

use super::{bytes::fetch_bytes, client::HttpClientTrait};
use crate::errors::RecorderError;

pub async fn fetch_image<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> Result<Bytes, RecorderError> {
    fetch_bytes(client, url).await
}
