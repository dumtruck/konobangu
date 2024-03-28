use bytes::Bytes;
pub use qbit_rs::model::{
    Torrent as QbitTorrent, TorrentContent as QbitTorrentContent,
    TorrentFilter as QbitTorrentFilter, TorrentSource as QbitTorrentSource,
};
use reqwest::IntoUrl;

pub(crate) async fn download_bytes<T: IntoUrl>(url: T) -> eyre::Result<Bytes> {
    let request_client = reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build()?;
    let bytes = request_client.get(url).send().await?.bytes().await?;
    Ok(bytes)
}

pub const BITTORRENT_MIME_TYPE: &str = "application/x-bittorrent";
pub const MAGNET_SCHEMA: &str = "magnet";
pub const DEFAULT_USER_AGENT: &str = "Wget/1.13.4 (linux-gnu)";
