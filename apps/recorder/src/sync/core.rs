use std::fmt::Debug;

use itertools::Itertools;
use lazy_static::lazy_static;
use librqbit_core::{
    magnet::Magnet,
    torrent_metainfo::{torrent_from_bytes, TorrentMetaV1Owned},
};
use quirks_path::{Path, PathBuf};
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{QbitTorrent, QbitTorrentContent, TorrentDownloadError};
use crate::fetch::{fetch_bytes, HttpClient};

pub const BITTORRENT_MIME_TYPE: &str = "application/x-bittorrent";
pub const MAGNET_SCHEMA: &str = "magnet";

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

lazy_static! {
    static ref TORRENT_HASH_RE: Regex = Regex::new(r"[a-fA-F0-9]{40}").unwrap();
    static ref TORRENT_EXT_RE: Regex = Regex::new(r"\.torrent$").unwrap();
}

#[derive(Clone, PartialEq, Eq)]
pub enum TorrentSource {
    MagnetUrl {
        url: Url,
        hash: String,
    },
    TorrentUrl {
        url: Url,
        hash: String,
    },
    TorrentFile {
        torrent: Vec<u8>,
        hash: String,
        name: Option<String>,
    },
}

impl TorrentSource {
    pub async fn parse(client: Option<&HttpClient>, url: &str) -> eyre::Result<Self> {
        let url = Url::parse(url)?;
        let source = if url.scheme() == MAGNET_SCHEMA {
            TorrentSource::from_magnet_url(url)?
        } else if let Some(basename) = url
            .clone()
            .path_segments()
            .and_then(|segments| segments.last())
        {
            if let (Some(match_hash), true) = (
                TORRENT_HASH_RE.find(basename),
                TORRENT_EXT_RE.is_match(basename),
            ) {
                TorrentSource::from_torrent_url(url, match_hash.as_str().to_string())?
            } else {
                let contents = fetch_bytes(client, url).await?;
                TorrentSource::from_torrent_file(contents.to_vec(), Some(basename.to_string()))?
            }
        } else {
            let contents = fetch_bytes(client, url).await?;
            TorrentSource::from_torrent_file(contents.to_vec(), None)?
        };
        Ok(source)
    }

    pub fn from_torrent_file(file: Vec<u8>, name: Option<String>) -> eyre::Result<Self> {
        let torrent: TorrentMetaV1Owned = torrent_from_bytes(&file)
            .map_err(|_| TorrentDownloadError::InvalidTorrentFileFormat)?;
        let hash = torrent.info_hash.as_string();
        Ok(TorrentSource::TorrentFile {
            torrent: file,
            hash,
            name,
        })
    }

    pub fn from_magnet_url(url: Url) -> eyre::Result<Self> {
        if url.scheme() != MAGNET_SCHEMA {
            Err(TorrentDownloadError::InvalidUrlSchema {
                found: url.scheme().to_string(),
                expected: MAGNET_SCHEMA.to_string(),
            }
            .into())
        } else {
            let magnet = Magnet::parse(url.as_str()).map_err(|_| {
                TorrentDownloadError::InvalidMagnetFormat {
                    url: url.as_str().to_string(),
                }
            })?;

            let hash = magnet
                .as_id20()
                .ok_or_else(|| TorrentDownloadError::InvalidMagnetFormat {
                    url: url.as_str().to_string(),
                })?
                .as_string();
            Ok(TorrentSource::MagnetUrl { url, hash })
        }
    }

    pub fn from_torrent_url(url: Url, hash: String) -> eyre::Result<Self> {
        Ok(TorrentSource::TorrentUrl { url, hash })
    }

    pub fn hash(&self) -> &str {
        match self {
            TorrentSource::MagnetUrl { hash, .. } => hash,
            TorrentSource::TorrentUrl { hash, .. } => hash,
            TorrentSource::TorrentFile { hash, .. } => hash,
        }
    }
}

impl Debug for TorrentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentSource::MagnetUrl { url, .. } => {
                write!(f, "MagnetUrl {{ url: {} }}", url.as_str())
            }
            TorrentSource::TorrentUrl { url, .. } => {
                write!(f, "TorrentUrl {{ url: {} }}", url.as_str())
            }
            TorrentSource::TorrentFile { name, hash, .. } => write!(
                f,
                "TorrentFile {{ name: \"{}\", hash: \"{hash}\" }}",
                name.as_deref().unwrap_or_default()
            ),
        }
    }
}

pub trait TorrentContent {
    fn get_name(&self) -> &str;

    fn get_all_size(&self) -> u64;

    fn get_progress(&self) -> f64;

    fn get_curr_size(&self) -> u64;
}

impl TorrentContent for QbitTorrentContent {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_all_size(&self) -> u64 {
        self.size
    }

    fn get_progress(&self) -> f64 {
        self.progress
    }

    fn get_curr_size(&self) -> u64 {
        u64::clamp(
            f64::round(self.get_all_size() as f64 * self.get_progress()) as u64,
            0,
            self.get_all_size(),
        )
    }
}

#[derive(Debug, Clone)]
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

    pub fn get_name(&self) -> Option<&str> {
        match self {
            Torrent::Qbit { torrent, .. } => torrent.name.as_deref(),
        }
    }

    pub fn get_hash(&self) -> Option<&str> {
        match self {
            Torrent::Qbit { torrent, .. } => torrent.hash.as_deref(),
        }
    }

    pub fn get_save_path(&self) -> Option<&str> {
        match self {
            Torrent::Qbit { torrent, .. } => torrent.save_path.as_deref(),
        }
    }

    pub fn get_content_path(&self) -> Option<&str> {
        match self {
            Torrent::Qbit { torrent, .. } => torrent.content_path.as_deref(),
        }
    }

    pub fn get_tags(&self) -> Vec<&str> {
        match self {
            Torrent::Qbit { torrent, .. } => torrent.tags.as_deref().map_or_else(Vec::new, |s| {
                s.split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect_vec()
            }),
        }
    }

    pub fn get_category(&self) -> Option<&str> {
        match self {
            Torrent::Qbit { torrent, .. } => torrent.category.as_deref(),
        }
    }
}

#[async_trait::async_trait]
pub trait TorrentDownloader {
    async fn get_torrents_info(
        &self,
        status_filter: TorrentFilter,
        category: Option<String>,
        tag: Option<String>,
    ) -> eyre::Result<Vec<Torrent>>;

    async fn add_torrents(
        &self,
        source: TorrentSource,
        save_path: String,
        category: Option<&str>,
    ) -> eyre::Result<()>;

    async fn delete_torrents(&self, hashes: Vec<String>) -> eyre::Result<()>;

    async fn rename_torrent_file(
        &self,
        hash: &str,
        old_path: &str,
        new_path: &str,
    ) -> eyre::Result<()>;

    async fn move_torrents(&self, hashes: Vec<String>, new_path: &str) -> eyre::Result<()>;

    async fn get_torrent_path(&self, hashes: String) -> eyre::Result<Option<String>>;

    async fn check_connection(&self) -> eyre::Result<()>;

    async fn set_torrents_category(&self, hashes: Vec<String>, category: &str) -> eyre::Result<()>;

    async fn add_torrent_tags(&self, hashes: Vec<String>, tags: Vec<String>) -> eyre::Result<()>;

    async fn add_category(&self, category: &str) -> eyre::Result<()>;

    fn get_save_path(&self, sub_path: &Path) -> PathBuf;
}
