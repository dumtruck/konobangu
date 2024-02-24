use bytes::Bytes;

pub async fn download_bytes (url: &str) -> eyre::Result<Bytes> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    Ok(bytes)
}