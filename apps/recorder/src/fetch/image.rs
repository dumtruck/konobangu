use bytes::Bytes;
use reqwest::IntoUrl;

use super::{bytes::fetch_bytes, client::HttpClientTrait};

pub async fn fetch_image<T: IntoUrl, H: HttpClientTrait>(
    client: &H,
    url: T,
) -> color_eyre::eyre::Result<Bytes> {
    fetch_bytes(client, url).await
}
