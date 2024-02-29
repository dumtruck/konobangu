use bytes::Bytes;
use reqwest::IntoUrl;

use super::bytes::download_bytes;

pub async fn download_image<U: IntoUrl>(url: U) -> eyre::Result<Bytes> {
    download_bytes(url).await
}
