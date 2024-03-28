use eyre::OptionExt;
use itertools::Itertools;
use lazy_static::lazy_static;
use librqbit_core::{
    magnet::Magnet,
    torrent_metainfo::{torrent_from_bytes, TorrentMetaV1Owned},
};
use quirks_path::{Path, PathBuf};
use regex::Regex;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, IntoActiveModel};
use serde::{Deserialize, Serialize};
use url::Url;

use super::{
    defs::{
        download_bytes, QbitTorrent, QbitTorrentContent, QbitTorrentFilter, QbitTorrentSource,
        MAGNET_SCHEMA,
    },
    error::DownloaderError,
    qbitorrent::QBittorrentDownloader,
};
use crate::{
    models::{bangumi, downloaders, downloaders::DownloaderCategory, downloads},
    path::torrent_path::gen_bangumi_sub_path,
};

lazy_static! {
    static ref TORRENT_HASH_RE: Regex = Regex::new(r"[a-fA-F0-9]{40}").unwrap();
    static ref TORRENT_EXT_RE: Regex = Regex::new(r"\.torrent$").unwrap();
}

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
    pub async fn parse(url: &str) -> eyre::Result<Self> {
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
                let contents = download_bytes(url).await?;
                TorrentSource::from_torrent_file(contents.to_vec(), Some(basename.to_string()))?
            }
        } else {
            let contents = download_bytes(url).await?;
            TorrentSource::from_torrent_file(contents.to_vec(), None)?
        };
        Ok(source)
    }

    pub fn from_torrent_file(file: Vec<u8>, name: Option<String>) -> eyre::Result<Self> {
        let torrent: TorrentMetaV1Owned =
            torrent_from_bytes(&file).map_err(|_| DownloaderError::InvalidTorrentFileFormat)?;
        let hash = torrent.info_hash.as_string();
        Ok(TorrentSource::TorrentFile {
            torrent: file,
            hash,
            name,
        })
    }

    pub fn from_magnet_url(url: Url) -> eyre::Result<Self> {
        if url.scheme() != MAGNET_SCHEMA {
            Err(DownloaderError::InvalidUrlSchema {
                found: url.scheme().to_string(),
                expected: MAGNET_SCHEMA.to_string(),
            }
            .into())
        } else {
            let magnet =
                Magnet::parse(url.as_str()).map_err(|_| DownloaderError::InvalidMagnetFormat {
                    url: url.as_str().to_string(),
                })?;
            let hash = magnet
                .as_id20()
                .ok_or_eyre("no info hash found")?
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

impl From<TorrentSource> for QbitTorrentSource {
    fn from(value: TorrentSource) -> Self {
        match value {
            TorrentSource::MagnetUrl { url, .. } => QbitTorrentSource::Urls {
                urls: qbit_rs::model::Sep::from([url]),
            },
            TorrentSource::TorrentUrl { url, .. } => QbitTorrentSource::Urls {
                urls: qbit_rs::model::Sep::from([url]),
            },
            TorrentSource::TorrentFile {
                torrent: torrents, ..
            } => QbitTorrentSource::TorrentFiles { torrents },
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

    async fn add_downloads_for_bangumi<'a, 'b>(
        &self,
        db: &'a DatabaseConnection,
        downloads: &[&downloads::Model],
        mut bangumi: bangumi::Model,
    ) -> eyre::Result<bangumi::Model> {
        if bangumi.save_path.is_none() {
            let gen_sub_path = gen_bangumi_sub_path(&bangumi);
            let mut bangumi_active = bangumi.into_active_model();
            bangumi_active.save_path = ActiveValue::Set(Some(gen_sub_path.to_string()));
            bangumi = bangumi_active.update(db).await?;
        }

        let sub_path = bangumi
            .save_path
            .as_ref()
            .unwrap_or_else(|| unreachable!("must have a sub path"));

        let mut torrent_urls = vec![];
        for m in downloads.iter() {
            torrent_urls.push(Url::parse(&m.url as &str)?);
        }

        // make sequence to prevent too fast to be banned
        for d in downloads.iter() {
            let source = TorrentSource::parse(&d.url).await?;
            self.add_torrents(source, sub_path.clone(), Some("bangumi"))
                .await?;
        }

        Ok(bangumi)
    }
}

pub async fn build_torrent_downloader_from_downloader_model(
    model: downloaders::Model,
) -> eyre::Result<Box<dyn TorrentDownloader>> {
    Ok(Box::new(match &model.category {
        DownloaderCategory::QBittorrent => {
            QBittorrentDownloader::from_downloader_model(model).await?
        }
    }))
}
