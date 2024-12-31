pub mod core;
pub mod error;
pub mod qbit;

pub use core::{Torrent, TorrentContent, TorrentDownloader, TorrentFilter, TorrentSource};

pub use error::TorrentDownloadError;
pub use qbit::{
    QBittorrentDownloader, QBittorrentDownloaderCreation, QbitTorrent, QbitTorrentContent,
    QbitTorrentFile, QbitTorrentFilter, QbitTorrentSource,
};
