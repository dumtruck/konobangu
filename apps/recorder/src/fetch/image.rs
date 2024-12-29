use bytes::Bytes;
use reqwest::IntoUrl;

use super::{
    bytes::{download_bytes, download_bytes_with_client},
    HttpClient,
};

pub async fn download_image<U: IntoUrl>(url: U) -> eyre::Result<Bytes> {
    download_bytes(url).await
}

pub async fn download_image_with_client<T: IntoUrl>(
    client: Option<&HttpClient>,
    url: T,
) -> eyre::Result<Bytes> {
    download_bytes_with_client(client, url).await
}
