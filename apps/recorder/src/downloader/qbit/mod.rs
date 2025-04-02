use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    io,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
pub use qbit_rs::model::{
    Torrent as QbitTorrent, TorrentContent as QbitTorrentContent, TorrentFile as QbitTorrentFile,
    TorrentFilter as QbitTorrentFilter, TorrentSource as QbitTorrentSource,
};
use qbit_rs::{
    Qbit,
    model::{
        AddTorrentArg, Credential, GetTorrentListArg, NonEmptyStr, Sep, State, TorrentFile,
        TorrentSource,
    },
};
use quirks_path::{Path, PathBuf};
use seaography::itertools::Itertools;
use snafu::prelude::*;
use tokio::sync::watch;
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
    pub categories: HashSet<String>,
    pub tags: HashSet<String>,
}

pub struct QBittorrentDownloader {
    pub subscriber_id: i32,
    pub downloader_id: i32,
    pub endpoint_url: Url,
    pub client: Arc<Qbit>,
    pub save_path: PathBuf,
    pub wait_sync_timeout: Duration,
    pub sync_watch: watch::Sender<DateTime<Utc>>,
    pub sync_data: QBittorrentSyncData,
}

impl QBittorrentDownloader {
    pub async fn from_creation(
        creation: QBittorrentDownloaderCreation,
    ) -> Result<Self, DownloaderError> {
        let endpoint_url = Url::parse(&creation.endpoint)?;

        let credential = Credential::new(creation.username, creation.password);

        let client = Qbit::new(endpoint_url.clone(), credential);

        client.login(false).await?;

        client.sync(None).await?;

        Ok(Self {
            client: Arc::new(client),
            endpoint_url,
            subscriber_id: creation.subscriber_id,
            save_path: creation.save_path.into(),
            wait_sync_timeout: Duration::from_millis(10000),
            downloader_id: creation.downloader_id,
            sync_watch: watch::channel(Utc::now()).0,
            sync_data: QBittorrentSyncData::default(),
        })
    }

    #[instrument(level = "debug")]
    pub async fn api_version(&self) -> Result<String, DownloaderError> {
        let result = self.client.get_webapi_version().await?;
        Ok(result)
    }

    pub async fn wait_sync_until<S>(
        &self,
        stop_wait_fn: S,
        timeout: Option<Duration>,
    ) -> Result<(), DownloaderError>
    where
        S: Fn(&QBittorrentSyncData) -> bool,
    {
        let mut receiver = self.sync_watch.subscribe();
        let timeout = timeout.unwrap_or(self.wait_sync_timeout);
        let start_time = Utc::now();

        while let Ok(()) = receiver.changed().await {
            let sync_time = receiver.borrow();
            if sync_time
                .signed_duration_since(start_time)
                .num_milliseconds()
                > timeout.as_millis() as i64
            {
                tracing::warn!(name = "wait_until timeout", timeout = ?timeout);
                return Err(DownloaderError::DownloadTimeoutError {
                    action: Cow::Borrowed("QBittorrentDownloader::wait_unit"),
                    timeout,
                });
            }
            if stop_wait_fn(&self.sync_data) {
                break;
            }
        }
        Ok(())
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
        self.wait_sync_until(|sync_data| sync_data.categories.contains(category), None)
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
        if !self.sync_data.categories.contains(category) {
            self.add_category(category).await?;
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
        let old_path = self
            .sync_data
            .torrents
            .get(hash)
            .and_then(|t| t.content_path.as_deref())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "no torrent or torrent does not contain content path",
                )
            })?
            .to_string();
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
        let tag = {
            let mut tags = vec![TORRENT_TAG_NAME.to_string()];
            tags.extend(creation.tags);
            Some(tags.into_iter().filter(|s| !s.is_empty()).join(","))
        };

        let save_path = Some(creation.save_path.into_string());

        let sources = creation.sources;
        let ids = HashSet::from_iter(sources.iter().map(|s| s.hash_info().to_string()));
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

        let category = TORRENT_TAG_NAME.to_string();

        if let Some(source) = urls_source {
            self.client
                .add_torrent(AddTorrentArg {
                    source,
                    savepath: save_path.clone(),
                    auto_torrent_management: Some(false),
                    category: Some(category.clone()),
                    tags: tag.clone(),
                    ..Default::default()
                })
                .await?;
        }

        if let Some(source) = files_source {
            self.client
                .add_torrent(AddTorrentArg {
                    source,
                    savepath: save_path.clone(),
                    auto_torrent_management: Some(false),
                    category: Some(category.clone()),
                    tags: tag,
                    ..Default::default()
                })
                .await?;
        }
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;
                ids.iter().all(|id| torrents.contains_key(id))
            },
            None,
        )
        .await?;
        Ok(ids)
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

    #[cfg(feature = "testcontainers")]
    pub async fn create_qbit_testcontainer()
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
        let _ = test_qbittorrent_downloader_impl(None, None).await;
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

        let image = create_qbit_testcontainer().await?;

        let container = image.start().await?;

        let mut logs = String::new();

        container.stdout(false).read_to_string(&mut logs).await?;

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
                if line.contains("A temporary password is provided for this session") {
                    line.split_whitespace().last()
                } else {
                    None
                }
            })
            .expect("should have password")
            .trim();

        tracing::info!(username, password);

        test_qbittorrent_downloader_impl(Some(username), Some(password)).await?;

        Ok(())
    }

    async fn test_qbittorrent_downloader_impl(
        username: Option<&str>,
        password: Option<&str>,
    ) -> RResult<()> {
        let http_client = build_testing_http_client()?;
        let base_save_path = Path::new(get_tmp_qbit_test_folder());

        let hash = "47ee2d69e7f19af783ad896541a07b012676f858".to_string();

        let mut downloader = QBittorrentDownloader::from_creation(QBittorrentDownloaderCreation {
            endpoint: "http://127.0.0.1:8080".to_string(),
            password: password.unwrap_or_default().to_string(),
            username: username.unwrap_or_default().to_string(),
            subscriber_id: 0,
            save_path: base_save_path.to_string(),
            downloader_id: 0,
        })
        .await?;

        downloader.wait_sync_timeout = Duration::from_secs(3);

        downloader.check_connection().await?;

        downloader
            .remove_torrents(vec![hash.clone()].into())
            .await?;

        let torrent_source = HashTorrentSource::from_url_and_http_client(
            &http_client,
            format!("https://mikanani.me/Download/20240301/{}.torrent", &hash),
        )
        .await?;

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
                    hash.clone(),
                )))
                .await?;

            let result = torrent_infos
                .into_iter()
                .find(|t| t.hash_info() == hash)
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
            .add_torrent_tags(vec![hash.clone()], vec![test_tag.clone()])
            .await?;

        let target_torrent = get_torrent().await?;

        assert!(target_torrent.tags().any(|s| s == test_tag));

        let test_category = format!("test_category_{}", Utc::now().timestamp());

        downloader
            .set_torrents_category(vec![hash.clone()], &test_category)
            .await?;

        let target_torrent = get_torrent().await?;

        assert_eq!(
            Some(test_category.as_str()),
            target_torrent.category().as_deref()
        );

        let moved_torrent_path = base_save_path.join(format!("moved_{}", Utc::now().timestamp()));

        downloader
            .move_torrents(vec![hash.clone()], moved_torrent_path.as_str())
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
            .move_torrent_contents(&hash, |f| {
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
            .remove_torrents(vec![hash.clone()].into())
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
