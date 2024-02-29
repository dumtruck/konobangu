pub use qbit_rs::model::{
    Torrent as QbitTorrent, TorrentContent as QbitTorrentContent,
    TorrentFilter as QbitTorrentFilter, TorrentSource as QbitTorrentSource,
};
use serde::{Deserialize, Serialize};
use url::Url;

pub const BITTORRENT_MIME_TYPE: &str = "application/x-bittorrent";
pub const DEFAULT_USER_AEGNT: &str = "Wget/1.13.4 (linux-gnu)";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TorrentFilter {
    All,
    Downloading,
    Completed,
    Paused,
    Active,
    Inactive,
    Resumed,
    Stalled,
    StalledUploading,
    StalledDownloading,
    Errored,
}

impl From<TorrentFilter> for QbitTorrentFilter {
    fn from(val: TorrentFilter) -> Self {
        match val {
            TorrentFilter::All => QbitTorrentFilter::All,
            TorrentFilter::Downloading => QbitTorrentFilter::Downloading,
            TorrentFilter::Completed => QbitTorrentFilter::Completed,
            TorrentFilter::Paused => QbitTorrentFilter::Paused,
            TorrentFilter::Active => QbitTorrentFilter::Active,
            TorrentFilter::Inactive => QbitTorrentFilter::Inactive,
            TorrentFilter::Resumed => QbitTorrentFilter::Resumed,
            TorrentFilter::Stalled => QbitTorrentFilter::Stalled,
            TorrentFilter::StalledUploading => QbitTorrentFilter::StalledUploading,
            TorrentFilter::StalledDownloading => QbitTorrentFilter::StalledDownloading,
            TorrentFilter::Errored => QbitTorrentFilter::Errored,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TorrentSources {
    Urls { urls: Vec<Url> },
    TorrentFiles { torrents: Vec<u8> },
}

impl From<TorrentSources> for QbitTorrentSource {
    fn from(value: TorrentSources) -> Self {
        match value {
            TorrentSources::Urls { urls } => QbitTorrentSource::Urls {
                urls: qbit_rs::model::Sep::from(urls),
            },
            TorrentSources::TorrentFiles { torrents } => {
                QbitTorrentSource::TorrentFiles { torrents }
            }
        }
    }
}

pub trait TorrentContent {
    fn get_name(&self) -> &str;
}

impl TorrentContent for QbitTorrentContent {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

pub enum Torrent {
    Qbit {
        torrent: QbitTorrent,
        contents: Vec<QbitTorrentContent>,
    },
}

impl Torrent {
    pub fn iter_files(&self) -> impl Iterator<Item = &dyn TorrentContent> {
        match self {
            Torrent::Qbit { contents, .. } => {
                contents.iter().map(|item| item as &dyn TorrentContent)
            }
        }
    }
}
