use bytes::Bytes;
use reqwest::IntoUrl;

use super::{bytes::fetch_bytes, HttpClient};

pub async fn fetch_image<T: IntoUrl>(client: Option<&HttpClient>, url: T) -> eyre::Result<Bytes> {
    fetch_bytes(client, url).await
}
