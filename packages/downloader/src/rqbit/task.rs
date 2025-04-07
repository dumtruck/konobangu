use std::{borrow::Cow, time::Duration};

use itertools::Itertools;
use qbit_rs::model::{
    GetTorrentListArg, State, Torrent as QbitTorrent, TorrentContent as QbitTorrentContent,
};
use quirks_path::{Path, PathBuf};

use crate::{
    DownloaderError,
    bittorrent::{
        source::HashTorrentSource,
        task::{SimpleTorrentHash, TorrentCreationTrait, TorrentStateTrait, TorrentTaskTrait},
    },
    core::{
        DownloadCreationTrait, DownloadIdSelector, DownloadSelectorTrait, DownloadStateTrait,
        DownloadTaskTrait,
    },
};

pub type RqbitHash = SimpleTorrentHash;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RqbitState(Option<State>);

impl DownloadStateTrait for RqbitState {}

impl TorrentStateTrait for RqbitState {}

impl From<Option<State>> for RqbitState {
    fn from(value: Option<State>) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct RqbitTask {
    pub hash_info: RqbitHash,
    pub torrent: QbitTorrent,
    pub contents: Vec<QbitTorrentContent>,
    pub state: RqbitState,
}

impl RqbitTask {
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
        let state = RqbitState::from(torrent.state.clone());
        Ok(Self {
            hash_info: hash,
            contents,
            state,
            torrent,
        })
    }
}

impl DownloadTaskTrait for RqbitTask {
    type State = RqbitState;
    type Id = RqbitHash;

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

impl TorrentTaskTrait for RqbitTask {
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
pub struct RqbitCreation {
    pub save_path: PathBuf,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub sources: Vec<HashTorrentSource>,
}

impl DownloadCreationTrait for RqbitCreation {
    type Task = RqbitTask;
}

impl TorrentCreationTrait for RqbitCreation {
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

pub type RqbitHashSelector = DownloadIdSelector<RqbitTask>;

pub struct RqbitComplexSelector {
    pub query: GetTorrentListArg,
}

impl From<RqbitHashSelector> for RqbitComplexSelector {
    fn from(value: RqbitHashSelector) -> Self {
        Self {
            query: GetTorrentListArg {
                hashes: Some(value.ids.join("|")),
                ..Default::default()
            },
        }
    }
}

impl DownloadSelectorTrait for RqbitComplexSelector {
    type Id = RqbitHash;
    type Task = RqbitTask;
}

pub enum RqbitSelector {
    Hash(RqbitHashSelector),
    Complex(RqbitComplexSelector),
}

impl DownloadSelectorTrait for RqbitSelector {
    type Id = RqbitHash;
    type Task = RqbitTask;

    fn try_into_ids_only(self) -> Result<Vec<Self::Id>, Self> {
        match self {
            RqbitSelector::Complex(c) => c.try_into_ids_only().map_err(RqbitSelector::Complex),
            RqbitSelector::Hash(h) => {
                let result = h
                    .try_into_ids_only()
                    .unwrap_or_else(|_| unreachable!("hash selector must contains hash"))
                    .into_iter();
                Ok(result.collect_vec())
            }
        }
    }
}
