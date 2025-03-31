pub mod core;
pub mod error;
pub mod qbit;
pub mod rqbit;
pub mod utils;

pub use core::{
    BITTORRENT_MIME_TYPE, MAGNET_SCHEMA, Torrent, TorrentContent, TorrentDownloader, TorrentFilter,
    TorrentSource,
};

pub use error::TorrentDownloadError;
pub use qbit::{
    QBittorrentDownloader, QBittorrentDownloaderCreation, QbitTorrent, QbitTorrentContent,
    QbitTorrentFile, QbitTorrentFilter, QbitTorrentSource,
};
