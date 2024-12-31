pub mod core;
pub mod error;
pub mod qbit;

pub use core::{
    Torrent, TorrentContent, TorrentDownloader, TorrentFilter, TorrentSource, BITTORRENT_MIME_TYPE,
    MAGNET_SCHEMA,
};

pub use error::TorrentDownloadError;
pub use qbit::{
    QBittorrentDownloader, QBittorrentDownloaderCreation, QbitTorrent, QbitTorrentContent,
    QbitTorrentFile, QbitTorrentFilter, QbitTorrentSource,
};
