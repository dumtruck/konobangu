use std::{borrow::Cow, time::Duration};

use itertools::Itertools;
use qbit_rs::model::{
    GetTorrentListArg, State, Torrent as QbitTorrent, TorrentContent as QbitTorrentContent,
};
use quirks_path::{Path, PathBuf};

use crate::downloader::{
    DownloaderError,
    bittorrent::{
        source::HashTorrentSource,
        task::{TorrentCreationTrait, TorrentHashTrait, TorrentStateTrait, TorrentTaskTrait},
    },
    core::{
        DownloadCreationTrait, DownloadIdSelector, DownloadIdTrait, DownloadSelectorTrait,
        DownloadStateTrait, DownloadTaskTrait,
    },
};

pub type QBittorrentHash = String;

impl DownloadIdTrait for QBittorrentHash {}

impl TorrentHashTrait for QBittorrentHash {}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QBittorrentState(Option<State>);

impl DownloadStateTrait for QBittorrentState {}

impl TorrentStateTrait for QBittorrentState {}

impl From<Option<State>> for QBittorrentState {
    fn from(value: Option<State>) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct QBittorrentTask {
    pub hash_info: QBittorrentHash,
    pub torrent: QbitTorrent,
    pub contents: Vec<QbitTorrentContent>,
    pub state: QBittorrentState,
}

impl QBittorrentTask {
    pub fn from_query(
        torrent: QbitTorrent,
        contents: Vec<QbitTorrentContent>,
    ) -> Result<Self, DownloaderError> {
        let hash = torrent
            .hash
            .clone()
            .ok_or_else(|| DownloaderError::TorrentMetaError {
                message: "missing hash".to_string(),
                source: None.into(),
            })?;
        let state = QBittorrentState(torrent.state.clone());
        Ok(Self {
            hash_info: hash,
            contents,
            state,
            torrent,
        })
    }
}

impl DownloadTaskTrait for QBittorrentTask {
    type State = QBittorrentState;
    type Id = QBittorrentHash;

    fn id(&self) -> &Self::Id {
        &self.hash_info
    }

    fn into_id(self) -> Self::Id {
        self.hash_info
    }

    fn name(&self) -> Cow<'_, str> {
        self.torrent
            .name
            .as_deref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| DownloadTaskTrait::name(self))
    }

    fn speed(&self) -> Option<u64> {
        self.torrent.dlspeed.and_then(|s| u64::try_from(s).ok())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dl_bytes(&self) -> Option<u64> {
        self.torrent.downloaded.and_then(|v| u64::try_from(v).ok())
    }

    fn total_bytes(&self) -> Option<u64> {
        self.torrent.size.and_then(|v| u64::try_from(v).ok())
    }

    fn left_bytes(&self) -> Option<u64> {
        self.torrent.amount_left.and_then(|v| u64::try_from(v).ok())
    }

    fn et(&self) -> Option<Duration> {
        self.torrent
            .time_active
            .and_then(|v| u64::try_from(v).ok())
            .map(Duration::from_secs)
    }

    fn eta(&self) -> Option<Duration> {
        self.torrent
            .eta
            .and_then(|v| u64::try_from(v).ok())
            .map(Duration::from_secs)
    }

    fn progress(&self) -> Option<f32> {
        self.torrent.progress.as_ref().map(|s| *s as f32)
    }
}

impl TorrentTaskTrait for QBittorrentTask {
    fn hash_info(&self) -> &str {
        &self.hash_info
    }

    fn tags(&self) -> impl Iterator<Item = Cow<'_, str>> {
        self.torrent
            .tags
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(Cow::Borrowed)
    }

    fn category(&self) -> Option<Cow<'_, str>> {
        self.torrent.category.as_deref().map(Cow::Borrowed)
    }
}

#[derive(Debug, Clone, Default)]
pub struct QBittorrentCreation {
    pub save_path: PathBuf,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub sources: Vec<HashTorrentSource>,
}

impl DownloadCreationTrait for QBittorrentCreation {
    type Task = QBittorrentTask;
}

impl TorrentCreationTrait for QBittorrentCreation {
    fn save_path(&self) -> &Path {
        self.save_path.as_ref()
    }

    fn save_path_mut(&mut self) -> &mut PathBuf {
        &mut self.save_path
    }

    fn sources_mut(&mut self) -> &mut Vec<HashTorrentSource> {
        &mut self.sources
    }
}

pub type QBittorrentHashSelector = DownloadIdSelector<QBittorrentTask>;

pub struct QBittorrentComplexSelector {
    pub query: GetTorrentListArg,
}

impl From<QBittorrentHashSelector> for QBittorrentComplexSelector {
    fn from(value: QBittorrentHashSelector) -> Self {
        Self {
            query: GetTorrentListArg {
                hashes: Some(value.ids.join("|")),
                ..Default::default()
            },
        }
    }
}

impl DownloadSelectorTrait for QBittorrentComplexSelector {
    type Id = QBittorrentHash;
    type Task = QBittorrentTask;
}

pub enum QBittorrentSelector {
    Hash(QBittorrentHashSelector),
    Complex(QBittorrentComplexSelector),
}

impl DownloadSelectorTrait for QBittorrentSelector {
    type Id = QBittorrentHash;
    type Task = QBittorrentTask;

    fn try_into_ids_only(self) -> Result<Vec<Self::Id>, Self> {
        match self {
            QBittorrentSelector::Complex(c) => {
                c.try_into_ids_only().map_err(QBittorrentSelector::Complex)
            }
            QBittorrentSelector::Hash(h) => {
                let result = h
                    .try_into_ids_only()
                    .unwrap_or_else(|_| unreachable!("hash selector must contains hash"))
                    .into_iter();
                Ok(result.collect_vec())
            }
        }
    }
}
