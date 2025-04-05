use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    io,
    sync::{Arc, Weak},
    time::Duration,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use itertools::Itertools;
use merge_struct::merge;
pub use qbit_rs::model::{
    Torrent as QbitTorrent, TorrentContent as QbitTorrentContent, TorrentFile as QbitTorrentFile,
    TorrentFilter as QbitTorrentFilter, TorrentSource as QbitTorrentSource,
};
use qbit_rs::{
    Qbit,
    model::{
        AddTorrentArg, Category, Credential, GetTorrentListArg, NonEmptyStr, Sep, State, SyncData,
        TorrentFile, TorrentSource,
    },
};
use quirks_path::{Path, PathBuf};
use snafu::prelude::*;
use tokio::{
    sync::{RwLock, watch},
    time::sleep,
};
use tracing::instrument;
use url::Url;

use super::{DownloaderError, utils::path_equals_as_file_url};
use crate::downloader::{
    bittorrent::{
        downloader::TorrentDownloaderTrait,
        source::{HashTorrentSource, HashTorrentSourceTrait, MagnetUrlSource, TorrentFileSource},
        task::{
            TORRENT_TAG_NAME, TorrentCreationTrait, TorrentHashTrait, TorrentStateTrait,
            TorrentTaskTrait,
        },
    },
    core::{
        DownloadCreationTrait, DownloadIdSelector, DownloadIdTrait, DownloadSelectorTrait,
        DownloadStateTrait, DownloadTaskTrait, DownloaderTrait,
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
    fn from_query(
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

pub struct QBittorrentDownloaderCreation {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub save_path: String,
    pub subscriber_id: i32,
    pub downloader_id: i32,
    pub wait_sync_timeout: Option<Duration>,
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

#[derive(Default)]
pub struct QBittorrentSyncData {
    pub torrents: HashMap<String, QbitTorrent>,
    pub categories: HashMap<String, Category>,
    pub tags: HashSet<String>,
    pub trackers: HashMap<String, Vec<String>>,
    pub server_state: HashMap<String, serde_value::Value>,
    pub rid: i64,
}
impl QBittorrentSyncData {
    pub fn patch(&mut self, data: SyncData) {
        self.rid = data.rid;
        if data.full_update.is_some_and(|s| s) {
            self.torrents.clear();
            self.categories.clear();
            self.tags.clear();
            self.trackers.clear();
        }
        if let Some(remove_categories) = data.categories_removed {
            for c in remove_categories {
                self.categories.remove(&c);
            }
        }
        if let Some(add_categories) = data.categories {
            self.categories.extend(add_categories);
        }
        if let Some(remove_tags) = data.tags_removed {
            for t in remove_tags {
                self.tags.remove(&t);
            }
        }
        if let Some(add_tags) = data.tags {
            self.tags.extend(add_tags);
        }
        if let Some(remove_torrents) = data.torrents_removed {
            for t in remove_torrents {
                self.torrents.remove(&t);
            }
        }
        if let Some(add_torrents) = data.torrents {
            for (hash, torrent_patch) in add_torrents {
                if let Some(torrent_full) = self.torrents.get_mut(&hash) {
                    *torrent_full = merge(torrent_full, &torrent_patch).unwrap_or_else(|_| {
                        unreachable!("failed to merge torrents, but they are same type")
                    });
                } else {
                    self.torrents.insert(hash, torrent_patch);
                }
            }
        }
        if let Some(remove_trackers) = data.trackers_removed {
            for t in remove_trackers {
                self.trackers.remove(&t);
            }
        }
        if let Some(add_trackers) = data.trackers {
            self.trackers.extend(add_trackers);
        }
        if let Some(server_state) = data.server_state {
            self.server_state = merge(&self.server_state, &server_state).unwrap_or_else(|_| {
                unreachable!("failed to merge server state, but they are same type")
            });
        }
    }
}

pub struct QBittorrentDownloader {
    pub subscriber_id: i32,
    pub downloader_id: i32,
    pub endpoint_url: Url,
    pub client: Arc<Qbit>,
    pub save_path: PathBuf,
    pub wait_sync_timeout: Duration,
    pub sync_watch: watch::Sender<DateTime<Utc>>,
    pub sync_data: Arc<RwLock<QBittorrentSyncData>>,
}

impl QBittorrentDownloader {
    pub async fn from_creation(
        creation: QBittorrentDownloaderCreation,
    ) -> Result<Arc<Self>, DownloaderError> {
        let endpoint_url = Url::parse(&creation.endpoint)?;

        let credential = Credential::new(creation.username, creation.password);

        let client = Qbit::new(endpoint_url.clone(), credential);

        client.login(false).await?;

        client.sync(None).await?;

        let downloader = Arc::new(Self {
            client: Arc::new(client),
            endpoint_url,
            subscriber_id: creation.subscriber_id,
            save_path: creation.save_path.into(),
            wait_sync_timeout: creation
                .wait_sync_timeout
                .unwrap_or(Duration::from_millis(10000)),
            downloader_id: creation.downloader_id,
            sync_watch: watch::channel(Utc::now()).0,
            sync_data: Arc::new(RwLock::new(QBittorrentSyncData::default())),
        });

        let event_loop_me = Arc::downgrade(&downloader);

        tokio::spawn(async move { Self::start_event_loop(event_loop_me).await });

        Ok(downloader)
    }

    async fn start_event_loop(me: Weak<Self>) {
        let mut tick = 0;

        loop {
            sleep(Duration::from_millis(100)).await;
            if let Some(me) = me.upgrade() {
                if tick >= 100 {
                    let _ = me.sync_data().await.inspect_err(|e| {
                        tracing::error!(name = "sync_data", error = ?e);
                    });
                    tick = 0;
                    continue;
                }
                let count = me.sync_watch.receiver_count();
                if count > 0 && tick >= 10 {
                    let _ = me.sync_data().await.inspect_err(|e| {
                        tracing::error!(name = "sync_data", error = ?e);
                    });
                    tick = i32::max(0, tick - 10);
                } else {
                    tick += 1;
                }
            }
        }
    }

    #[instrument(level = "debug")]
    pub async fn api_version(&self) -> Result<String, DownloaderError> {
        let result = self.client.get_webapi_version().await?;
        Ok(result)
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn add_category(&self, category: &str) -> Result<(), DownloaderError> {
        self.client
            .add_category(
                NonEmptyStr::new(category)
                    .whatever_context::<_, DownloaderError>("category can not be empty")?,
                self.save_path.as_str(),
            )
            .await?;
        self.wait_sync_until(
            |sync_data| sync_data.categories.contains_key(category),
            None,
        )
        .await?;

        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn check_connection(&self) -> Result<(), DownloaderError> {
        self.api_version().await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn set_torrents_category(
        &self,
        hashes: Vec<String>,
        category: &str,
    ) -> Result<(), DownloaderError> {
        {
            let sync_data = self.sync_data.read().await;
            if !sync_data.categories.contains_key(category) {
                self.add_category(category).await?;
            }
        }
        self.client
            .set_torrent_category(hashes.clone(), category)
            .await?;
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;
                hashes.iter().all(|h| {
                    torrents
                        .get(h)
                        .is_some_and(|t| t.category.as_deref().is_some_and(|c| c == category))
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    pub fn get_save_path(&self, sub_path: &Path) -> PathBuf {
        self.save_path.join(sub_path)
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn add_torrent_tags(
        &self,
        hashes: Vec<String>,
        tags: Vec<String>,
    ) -> Result<(), DownloaderError> {
        if tags.is_empty() {
            whatever!("add bittorrent tags can not be empty");
        }
        self.client
            .add_torrent_tags(hashes.clone(), tags.clone())
            .await?;
        let tag_sets = tags.iter().map(|s| s.as_str()).collect::<HashSet<&str>>();
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;

                hashes.iter().all(|h| {
                    torrents.get(h).is_some_and(|t| {
                        t.tags.as_ref().is_some_and(|t| {
                            t.split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .collect::<HashSet<&str>>()
                                .is_superset(&tag_sets)
                        })
                    })
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self, replacer))]
    pub async fn move_torrent_contents<F: FnOnce(String) -> String>(
        &self,
        hash: &str,
        replacer: F,
    ) -> Result<(), DownloaderError> {
        let old_path = {
            let sync_data = self.sync_data.read().await;
            sync_data
                .torrents
                .get(hash)
                .and_then(|t| t.content_path.as_deref())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "no torrent or torrent does not contain content path",
                    )
                })?
                .to_string()
        };
        let new_path = replacer(old_path.clone());
        self.client
            .rename_file(hash, old_path.clone(), new_path.to_string())
            .await?;
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;
                torrents.get(hash).is_some_and(|t| {
                    t.content_path.as_deref().is_some_and(|p| {
                        path_equals_as_file_url(p, &new_path)
                            .inspect_err(|error| {
                                tracing::warn!(name = "path_equals_as_file_url", error = ?error);
                            })
                            .unwrap_or(false)
                    })
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn move_torrents(
        &self,
        hashes: Vec<String>,
        new_path: &str,
    ) -> Result<(), DownloaderError> {
        self.client
            .set_torrent_location(hashes.clone(), new_path)
            .await?;

        self.wait_sync_until(
            |sync_data| -> bool {
                let torrents = &sync_data.torrents;

                hashes.iter().all(|h| {
                    torrents.get(h).is_some_and(|t| {
                        t.save_path.as_deref().is_some_and(|p| {
                            path_equals_as_file_url(p, new_path)
                            .inspect_err(|error| {
                                tracing::warn!(name = "path_equals_as_file_url", error = ?error);
                            })
                            .unwrap_or(false)
                        })
                    })
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn get_torrent_path(
        &self,
        hashes: String,
    ) -> Result<Option<String>, DownloaderError> {
        let mut torrent_list = self
            .client
            .get_torrent_list(GetTorrentListArg {
                hashes: Some(hashes),
                ..Default::default()
            })
            .await?;
        let torrent = torrent_list
            .first_mut()
            .whatever_context::<_, DownloaderError>("No bittorrent found")?;
        Ok(torrent.save_path.take())
    }

    #[instrument(level = "debug", skip(self))]
    async fn sync_data(&self) -> Result<(), DownloaderError> {
        let rid = { self.sync_data.read().await.rid };
        let sync_data_patch = self.client.sync(Some(rid)).await?;
        {
            let mut sync_data = self.sync_data.write().await;
            sync_data.patch(sync_data_patch);
        }
        let now = Utc::now();
        self.sync_watch.send_replace(now);
        Ok(())
    }

    async fn wait_sync_until<S>(
        &self,
        stop_wait_fn: S,
        timeout: Option<Duration>,
    ) -> Result<(), DownloaderError>
    where
        S: Fn(&QBittorrentSyncData) -> bool,
    {
        let timeout = timeout.unwrap_or(self.wait_sync_timeout);
        let start_time = Utc::now();

        let mut receiver = self.sync_watch.subscribe();
        while let Ok(()) = receiver.changed().await {
            let has_timeout = {
                let sync_time = *receiver.borrow();
                sync_time
                    .signed_duration_since(start_time)
                    .num_milliseconds()
                    > timeout.as_millis() as i64
            };
            if has_timeout {
                tracing::warn!(name = "wait_until timeout", timeout = ?timeout);
                return Err(DownloaderError::DownloadTimeoutError {
                    action: Cow::Borrowed("QBittorrentDownloader::wait_unit"),
                    timeout,
                });
            }
            {
                let sync_data = &self.sync_data.read().await;
                if stop_wait_fn(sync_data) {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl DownloaderTrait for QBittorrentDownloader {
    type State = QBittorrentState;
    type Id = QBittorrentHash;
    type Task = QBittorrentTask;
    type Creation = QBittorrentCreation;
    type Selector = QBittorrentSelector;

    async fn add_downloads(
        &self,
        creation: Self::Creation,
    ) -> Result<HashSet<Self::Id>, DownloaderError> {
        let tags = {
            let mut tags = vec![TORRENT_TAG_NAME.to_string()];
            tags.extend(creation.tags);
            Some(tags.into_iter().filter(|s| !s.is_empty()).join(","))
        };

        let save_path = Some(creation.save_path.into_string());

        let sources = creation.sources;
        let hashes = HashSet::from_iter(sources.iter().map(|s| s.hash_info().to_string()));
        let (urls_source, files_source) = {
            let mut urls = vec![];
            let mut files = vec![];
            for s in sources {
                match s {
                    HashTorrentSource::MagnetUrl(MagnetUrlSource { url, .. }) => {
                        urls.push(Url::parse(&url)?)
                    }
                    HashTorrentSource::TorrentFile(TorrentFileSource {
                        payload, filename, ..
                    }) => files.push(TorrentFile {
                        filename,
                        data: payload.into(),
                    }),
                }
            }
            (
                if urls.is_empty() {
                    None
                } else {
                    Some(TorrentSource::Urls {
                        urls: Sep::from(urls),
                    })
                },
                if files.is_empty() {
                    None
                } else {
                    Some(TorrentSource::TorrentFiles { torrents: files })
                },
            )
        };

        let category = creation.category;

        if let Some(category) = category.as_deref() {
            let has_caetgory = {
                self.sync_data
                    .read()
                    .await
                    .categories
                    .contains_key(category)
            };
            if !has_caetgory {
                self.add_category(category).await?;
            }
        }

        if let Some(source) = urls_source {
            self.client
                .add_torrent(AddTorrentArg {
                    source,
                    savepath: save_path.clone(),
                    auto_torrent_management: Some(false),
                    category: category.clone(),
                    tags: tags.clone(),
                    ..Default::default()
                })
                .await?;
        }

        if let Some(source) = files_source {
            self.client
                .add_torrent(AddTorrentArg {
                    source,
                    savepath: save_path,
                    auto_torrent_management: Some(false),
                    category,
                    tags,
                    ..Default::default()
                })
                .await?;
        }
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;
                hashes.iter().all(|hash| torrents.contains_key(hash))
            },
            None,
        )
        .await?;
        Ok(hashes)
    }

    async fn pause_downloads(
        &self,
        selector: Self::Selector,
    ) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
        <Self as TorrentDownloaderTrait>::pause_downloads(self, selector).await
    }

    async fn resume_downloads(
        &self,
        selector: Self::Selector,
    ) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
        <Self as TorrentDownloaderTrait>::resume_downloads(self, selector).await
    }

    async fn remove_downloads(
        &self,
        selector: Self::Selector,
    ) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
        <Self as TorrentDownloaderTrait>::remove_downloads(self, selector).await
    }

    async fn query_downloads(
        &self,
        selector: QBittorrentSelector,
    ) -> Result<Vec<Self::Task>, DownloaderError> {
        let selector = match selector {
            QBittorrentSelector::Hash(h) => h.into(),
            QBittorrentSelector::Complex(c) => c,
        };

        let torrent_list = self.client.get_torrent_list(selector.query).await?;

        let torrent_contents = try_join_all(torrent_list.iter().map(|s| async {
            if let Some(hash) = &s.hash {
                self.client.get_torrent_contents(hash as &str, None).await
            } else {
                Ok(vec![])
            }
        }))
        .await?;

        let tasks = torrent_list
            .into_iter()
            .zip(torrent_contents)
            .map(|(t, c)| Self::Task::from_query(t, c))
            .collect::<Result<Vec<Self::Task>, _>>()?;
        Ok(tasks)
    }
}

#[async_trait]
impl TorrentDownloaderTrait for QBittorrentDownloader {
    type IdSelector = DownloadIdSelector<Self::Task>;
    #[instrument(level = "debug", skip(self))]
    async fn pause_torrents(
        &self,
        hashes: Self::IdSelector,
    ) -> Result<Self::IdSelector, DownloaderError> {
        self.client.pause_torrents(hashes.clone()).await?;
        Ok(hashes)
    }

    #[instrument(level = "debug", skip(self))]
    async fn resume_torrents(
        &self,
        hashes: Self::IdSelector,
    ) -> Result<Self::IdSelector, DownloaderError> {
        self.client.resume_torrents(hashes.clone()).await?;
        Ok(hashes)
    }

    #[instrument(level = "debug", skip(self))]
    async fn remove_torrents(
        &self,
        hashes: Self::IdSelector,
    ) -> Result<Self::IdSelector, DownloaderError> {
        self.client
            .delete_torrents(hashes.clone(), Some(true))
            .await?;
        self.wait_sync_until(
            |sync_data| -> bool {
                let torrents = &sync_data.torrents;
                hashes.iter().all(|h| !torrents.contains_key(h))
            },
            None,
        )
        .await?;
        Ok(hashes)
    }
}

impl Debug for QBittorrentDownloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QBittorrentDownloader")
            .field("subscriber_id", &self.subscriber_id)
            .field("client", &self.endpoint_url.as_str())
            .finish()
    }
}

#[cfg(test)]
pub mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        downloader::core::DownloadIdSelectorTrait,
        errors::{RError, app_error::RResult},
        test_utils::fetch::build_testing_http_client,
    };

    fn get_tmp_qbit_test_folder() -> &'static str {
        if cfg!(all(windows, not(feature = "testcontainers"))) {
            "C:\\Windows\\Temp\\konobangu\\qbit"
        } else {
            "/tmp/konobangu/qbit"
        }
    }

    #[derive(Serialize)]
    struct MockFileItem {
        path: String,
        size: u64,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct MockRequest {
        id: String,
        file_list: Vec<MockFileItem>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    pub struct MockResponse {
        torrent_url: String,
        magnet_url: String,
        hash: String,
    }

    #[cfg(feature = "testcontainers")]
    pub async fn create_torrents_testcontainers()
    -> RResult<testcontainers::ContainerRequest<testcontainers::GenericImage>> {
        use testcontainers::{
            GenericImage,
            core::{ContainerPort, WaitFor},
        };
        use testcontainers_modules::testcontainers::ImageExt;

        use crate::test_utils::testcontainers::ContainerRequestEnhancedExt;

        let container = GenericImage::new("ghcr.io/dumtruck/konobangu-testing-torrents", "latest")
            .with_wait_for(WaitFor::message_on_stdout("Listening on"))
            .with_mapped_port(6080, ContainerPort::Tcp(6080))
            .with_mapped_port(6081, ContainerPort::Tcp(6081))
            .with_mapped_port(6082, ContainerPort::Tcp(6082))
            // .with_reuse(ReuseDirective::Always)
            .with_default_log_consumer()
            .with_prune_existed_label("konobangu-testing-torrents", true, true)
            .await?;

        Ok(container)
    }

    #[cfg(feature = "testcontainers")]
    pub async fn create_qbit_testcontainers()
    -> RResult<testcontainers::ContainerRequest<testcontainers::GenericImage>> {
        use testcontainers::{
            GenericImage,
            core::{
                ContainerPort,
                // ReuseDirective,
                WaitFor,
            },
        };
        use testcontainers_modules::testcontainers::ImageExt;

        use crate::test_utils::testcontainers::ContainerRequestEnhancedExt;

        let container = GenericImage::new("linuxserver/qbittorrent", "latest")
            .with_wait_for(WaitFor::message_on_stderr("Connection to localhost"))
            .with_env_var("WEBUI_PORT", "8080")
            .with_env_var("TZ", "Asia/Singapore")
            .with_env_var("TORRENTING_PORT", "6881")
            .with_mapped_port(6881, ContainerPort::Tcp(6881))
            .with_mapped_port(8080, ContainerPort::Tcp(8080))
            // .with_reuse(ReuseDirective::Always)
            .with_default_log_consumer()
            .with_prune_existed_label("qbit-downloader", true, true)
            .await?;

        Ok(container)
    }

    #[cfg(not(feature = "testcontainers"))]
    #[tokio::test]
    async fn test_qbittorrent_downloader() {
        let hash = "47ee2d69e7f19af783ad896541a07b012676f858".to_string();
        let torrent_url = "https://mikanani.me/Download/20240301/{}.torrent";
        let _ = test_qbittorrent_downloader_impl(torrent_url, hash, None, None).await;
    }

    #[cfg(feature = "testcontainers")]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_qbittorrent_downloader() -> RResult<()> {
        use testcontainers::runners::AsyncRunner;
        use tokio::io::AsyncReadExt;

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .init();

        let torrents_image = create_torrents_testcontainers().await?;
        let torrents_container = torrents_image.start().await?;

        let torrents_req = MockRequest {
            id: "f10ebdda-dd2e-43f8-b80c-bf0884d071c4".into(),
            file_list: vec![MockFileItem {
                path: "[Nekomoe kissaten&LoliHouse] Boku no Kokoro no Yabai Yatsu - 20 [WebRip \
                       1080p HEVC-10bit AAC ASSx2].mkv"
                    .into(),
                size: 1024,
            }],
        };

        let torrent_res: MockResponse = reqwest::Client::new()
            .post("http://127.0.0.1:6080/api/torrents/mock")
            .json(&torrents_req)
            .send()
            .await?
            .json()
            .await?;

        let qbit_image = create_qbit_testcontainers().await?;
        let qbit_container = qbit_image.start().await?;

        let mut logs = String::new();

        qbit_container
            .stdout(false)
            .read_to_string(&mut logs)
            .await?;

        let username = logs
            .lines()
            .find_map(|line| {
                if line.contains("The WebUI administrator username is") {
                    line.split_whitespace().last()
                } else {
                    None
                }
            })
            .expect("should have username")
            .trim();

        let password = logs
            .lines()
            .find_map(|line| {
                if line.contains("A temporary password is provided for") {
                    line.split_whitespace().last()
                } else {
                    None
                }
            })
            .expect("should have password")
            .trim();

        tracing::info!(username, password);

        test_qbittorrent_downloader_impl(
            torrent_res.torrent_url,
            torrent_res.hash,
            Some(username),
            Some(password),
        )
        .await?;

        torrents_container.stop().await?;

        Ok(())
    }

    async fn test_qbittorrent_downloader_impl(
        torrent_url: String,
        torrent_hash: String,
        username: Option<&str>,
        password: Option<&str>,
    ) -> RResult<()> {
        let http_client = build_testing_http_client()?;
        let base_save_path = Path::new(get_tmp_qbit_test_folder());

        let downloader = QBittorrentDownloader::from_creation(QBittorrentDownloaderCreation {
            endpoint: "http://127.0.0.1:8080".to_string(),
            password: password.unwrap_or_default().to_string(),
            username: username.unwrap_or_default().to_string(),
            subscriber_id: 0,
            save_path: base_save_path.to_string(),
            downloader_id: 0,
            wait_sync_timeout: Some(Duration::from_secs(3)),
        })
        .await?;

        downloader.check_connection().await?;

        downloader
            .remove_torrents(vec![torrent_hash.clone()].into())
            .await?;

        let torrent_source =
            HashTorrentSource::from_url_and_http_client(&http_client, torrent_url).await?;

        let folder_name = format!("torrent_test_{}", Utc::now().timestamp());
        let save_path = base_save_path.join(&folder_name);

        let torrent_creation = QBittorrentCreation {
            save_path,
            tags: vec![],
            sources: vec![torrent_source],
            category: None,
        };

        downloader.add_downloads(torrent_creation).await?;

        let get_torrent = async || -> Result<QBittorrentTask, DownloaderError> {
            let torrent_infos = downloader
                .query_downloads(QBittorrentSelector::Hash(QBittorrentHashSelector::from_id(
                    torrent_hash.clone(),
                )))
                .await?;

            let result = torrent_infos
                .into_iter()
                .find(|t| t.hash_info() == torrent_hash)
                .whatever_context::<_, DownloaderError>("no bittorrent")?;

            Ok(result)
        };

        let target_torrent = get_torrent().await?;

        let files = target_torrent.contents;

        assert!(!files.is_empty());

        let first_file = files.first().expect("should have first file");
        assert_eq!(
            &first_file.name,
            r#"[Nekomoe kissaten&LoliHouse] Boku no Kokoro no Yabai Yatsu - 20 [WebRip 1080p HEVC-10bit AAC ASSx2].mkv"#
        );

        let test_tag = format!("test_tag_{}", Utc::now().timestamp());

        downloader
            .add_torrent_tags(vec![torrent_hash.clone()], vec![test_tag.clone()])
            .await?;

        let target_torrent = get_torrent().await?;

        assert!(target_torrent.tags().any(|s| s == test_tag));

        let test_category = format!("test_category_{}", Utc::now().timestamp());

        downloader
            .set_torrents_category(vec![torrent_hash.clone()], &test_category)
            .await?;

        let target_torrent = get_torrent().await?;

        assert_eq!(
            Some(test_category.as_str()),
            target_torrent.category().as_deref()
        );

        let moved_torrent_path = base_save_path.join(format!("moved_{}", Utc::now().timestamp()));

        downloader
            .move_torrents(vec![torrent_hash.clone()], moved_torrent_path.as_str())
            .await?;

        let target_torrent = get_torrent().await?;

        let actual_content_path = &target_torrent
            .torrent
            .save_path
            .expect("failed to get actual save path");

        assert!(
            path_equals_as_file_url(actual_content_path, moved_torrent_path)
                .whatever_context::<_, RError>(
                    "failed to compare actual torrent path and found expected torrent path"
                )?
        );

        downloader
            .move_torrent_contents(&torrent_hash, |f| {
                f.replace(&folder_name, &format!("moved_{}", &folder_name))
            })
            .await?;

        let target_torrent = get_torrent().await?;

        let actual_content_path = &target_torrent
            .torrent
            .content_path
            .expect("failed to get actual content path");

        assert!(
            path_equals_as_file_url(
                actual_content_path,
                base_save_path.join(actual_content_path)
            )
            .whatever_context::<_, RError>(
                "failed to compare actual content path and found expected content path"
            )?
        );

        downloader
            .remove_torrents(vec![torrent_hash.clone()].into())
            .await?;

        let torrent_infos1 = downloader
            .query_downloads(QBittorrentSelector::Complex(QBittorrentComplexSelector {
                query: GetTorrentListArg::builder()
                    .filter(QbitTorrentFilter::All)
                    .build(),
            }))
            .await?;

        assert!(torrent_infos1.is_empty());

        Ok(())
    }
}
