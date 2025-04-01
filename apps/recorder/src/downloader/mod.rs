pub mod core;
pub mod errors;
pub mod qbit;
pub mod rqbit;
pub mod utils;

pub use core::{
    Torrent, TorrentContent, TorrentDownloader, TorrentFilter, TorrentSource, BITTORRENT_MIME_TYPE,
    MAGNET_SCHEMA,
};

pub use errors::DownloaderError;
pub use qbit::{
    QBittorrentDownloader, QBittorrentDownloaderCreation, QbitTorrent, QbitTorrentContent,
    QbitTorrentFile, QbitTorrentFilter, QbitTorrentSource,
};
