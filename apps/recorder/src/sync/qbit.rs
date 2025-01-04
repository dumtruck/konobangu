use std::{
    borrow::Cow, collections::HashSet, fmt::Debug, future::Future, sync::Arc, time::Duration,
};

use async_trait::async_trait;
use eyre::OptionExt;
use futures::future::try_join_all;
pub use qbit_rs::model::{
    Torrent as QbitTorrent, TorrentContent as QbitTorrentContent, TorrentFile as QbitTorrentFile,
    TorrentFilter as QbitTorrentFilter, TorrentSource as QbitTorrentSource,
};
use qbit_rs::{
    model::{AddTorrentArg, Credential, GetTorrentListArg, NonEmptyStr, SyncData},
    Qbit,
};
use quirks_path::{path_equals_as_file_url, Path, PathBuf};
use tokio::time::sleep;
use tracing::instrument;
use url::Url;

use super::{Torrent, TorrentDownloadError, TorrentDownloader, TorrentFilter, TorrentSource};

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
                torrent: torrents,
                name,
                ..
            } => QbitTorrentSource::TorrentFiles {
                torrents: vec![QbitTorrentFile {
                    filename: name.unwrap_or_default(),
                    data: torrents,
                }],
            },
        }
    }
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

pub struct QBittorrentDownloaderCreation {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub save_path: String,
    pub subscriber_id: i32,
}

pub struct QBittorrentDownloader {
    pub subscriber_id: i32,
    pub endpoint_url: Url,
    pub client: Arc<Qbit>,
    pub save_path: PathBuf,
    pub wait_sync_timeout: Duration,
}

impl QBittorrentDownloader {
    pub async fn from_creation(
        creation: QBittorrentDownloaderCreation,
    ) -> Result<Self, TorrentDownloadError> {
        let endpoint_url =
            Url::parse(&creation.endpoint).map_err(TorrentDownloadError::InvalidUrlParse)?;

        let credential = Credential::new(creation.username, creation.password);

        let client = Qbit::new(endpoint_url.clone(), credential);

        client
            .login(false)
            .await
            .map_err(TorrentDownloadError::QBitAPIError)?;

        client.sync(None).await?;

        Ok(Self {
            client: Arc::new(client),
            endpoint_url,
            subscriber_id: creation.subscriber_id,
            save_path: creation.save_path.into(),
            wait_sync_timeout: Duration::from_millis(10000),
        })
    }

    #[instrument(level = "debug")]
    pub async fn api_version(&self) -> eyre::Result<String> {
        let result = self.client.get_webapi_version().await?;
        Ok(result)
    }

    pub async fn wait_until<G, Fut, F, D, H, E>(
        &self,
        capture_fn: H,
        fetch_data_fn: G,
        mut stop_wait_fn: F,
        timeout: Option<Duration>,
    ) -> eyre::Result<()>
    where
        H: FnOnce() -> E,
        G: Fn(Arc<Qbit>, E) -> Fut,
        Fut: Future<Output = eyre::Result<D>>,
        F: FnMut(&D) -> bool,
        E: Clone,
        D: Debug + serde::Serialize,
    {
        let mut next_wait_ms = 32u64;
        let mut all_wait_ms = 0u64;
        let timeout = timeout.unwrap_or(self.wait_sync_timeout);
        let env = capture_fn();
        loop {
            sleep(Duration::from_millis(next_wait_ms)).await;
            all_wait_ms += next_wait_ms;
            if all_wait_ms >= timeout.as_millis() as u64 {
                // full update
                let sync_data = fetch_data_fn(self.client.clone(), env.clone()).await?;
                if stop_wait_fn(&sync_data) {
                    break;
                } else {
                    tracing::warn!(name = "wait_until timeout", sync_data = serde_json::to_string(&sync_data).unwrap(), timeout = ?timeout);
                    return Err(TorrentDownloadError::TimeoutError {
                        action: Cow::Borrowed("QBittorrentDownloader::wait_unit"),
                        timeout,
                    }
                    .into());
                }
            }
            let sync_data = fetch_data_fn(self.client.clone(), env.clone()).await?;
            if stop_wait_fn(&sync_data) {
                break;
            }
            next_wait_ms *= 2;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(self, stop_wait_fn))]
    pub async fn wait_torrents_until<F>(
        &self,
        arg: GetTorrentListArg,
        stop_wait_fn: F,
        timeout: Option<Duration>,
    ) -> eyre::Result<()>
    where
        F: FnMut(&Vec<QbitTorrent>) -> bool,
    {
        self.wait_until(
            || arg,
            async move |client: Arc<Qbit>,
                        arg: GetTorrentListArg|
                        -> eyre::Result<Vec<QbitTorrent>> {
                let data = client.get_torrent_list(arg).await?;
                Ok(data)
            },
            stop_wait_fn,
            timeout,
        )
        .await
    }

    #[instrument(level = "debug", skip(self, stop_wait_fn))]
    pub async fn wait_sync_until<F: FnMut(&SyncData) -> bool>(
        &self,
        stop_wait_fn: F,
        timeout: Option<Duration>,
    ) -> eyre::Result<()> {
        self.wait_until(
            || (),
            async move |client: Arc<Qbit>, _| -> eyre::Result<SyncData> {
                let data = client.sync(None).await?;
                Ok(data)
            },
            stop_wait_fn,
            timeout,
        )
        .await
    }

    #[instrument(level = "debug", skip(self, stop_wait_fn))]
    async fn wait_torrent_contents_until<F: FnMut(&Vec<QbitTorrentContent>) -> bool>(
        &self,
        hash: &str,
        stop_wait_fn: F,
        timeout: Option<Duration>,
    ) -> eyre::Result<()> {
        self.wait_until(
            || Arc::new(hash.to_string()),
            async move |client: Arc<Qbit>,
                        hash_arc: Arc<String>|
                        -> eyre::Result<Vec<QbitTorrentContent>> {
                let data = client.get_torrent_contents(hash_arc.as_str(), None).await?;
                Ok(data)
            },
            stop_wait_fn,
            timeout,
        )
        .await
    }
}

#[async_trait]
impl TorrentDownloader for QBittorrentDownloader {
    #[instrument(level = "debug", skip(self))]
    async fn get_torrents_info(
        &self,
        status_filter: TorrentFilter,
        category: Option<String>,
        tag: Option<String>,
    ) -> eyre::Result<Vec<Torrent>> {
        let arg = GetTorrentListArg {
            filter: Some(status_filter.into()),
            category,
            tag,
            ..Default::default()
        };
        let torrent_list = self.client.get_torrent_list(arg).await?;
        let torrent_contents = try_join_all(torrent_list.iter().map(|s| async {
            if let Some(hash) = &s.hash {
                self.client.get_torrent_contents(hash as &str, None).await
            } else {
                Ok(vec![])
            }
        }))
        .await?;
        Ok(torrent_list
            .into_iter()
            .zip(torrent_contents)
            .map(|(torrent, contents)| Torrent::Qbit { torrent, contents })
            .collect::<Vec<_>>())
    }

    #[instrument(level = "debug", skip(self))]
    async fn add_torrents(
        &self,
        source: TorrentSource,
        save_path: String,
        category: Option<&str>,
    ) -> eyre::Result<()> {
        let arg = AddTorrentArg {
            source: source.clone().into(),
            savepath: Some(save_path),
            category: category.map(String::from),
            auto_torrent_management: Some(false),
            ..Default::default()
        };
        let add_result = self.client.add_torrent(arg.clone()).await;
        if let (
            Err(qbit_rs::Error::ApiError(qbit_rs::ApiError::CategoryNotFound)),
            Some(category),
        ) = (&add_result, category)
        {
            self.add_category(category).await?;
            self.client.add_torrent(arg).await?;
        } else {
            add_result?;
        }
        let source_hash = source.hash();
        self.wait_sync_until(
            |sync_data| {
                sync_data
                    .torrents
                    .as_ref()
                    .is_some_and(|t| t.contains_key(source_hash))
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    async fn delete_torrents(&self, hashes: Vec<String>) -> eyre::Result<()> {
        self.client
            .delete_torrents(hashes.clone(), Some(true))
            .await?;
        self.wait_torrents_until(
            GetTorrentListArg::builder()
                .hashes(hashes.join("|"))
                .build(),
            |torrents| -> bool { torrents.is_empty() },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    async fn rename_torrent_file(
        &self,
        hash: &str,
        old_path: &str,
        new_path: &str,
    ) -> eyre::Result<()> {
        self.client.rename_file(hash, old_path, new_path).await?;
        let new_path = self.save_path.join(new_path);
        let save_path = self.save_path.as_path();
        self.wait_torrent_contents_until(
            hash,
            |contents| -> bool {
                contents.iter().any(|c| {
                    path_equals_as_file_url(save_path.join(&c.name), &new_path)
                        .inspect_err(|error| {
                            tracing::warn!(name = "path_equals_as_file_url", error = ?error);
                        })
                        .unwrap_or(false)
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    async fn move_torrents(&self, hashes: Vec<String>, new_path: &str) -> eyre::Result<()> {
        self.client
            .set_torrent_location(hashes.clone(), new_path)
            .await?;

        self.wait_torrents_until(
            GetTorrentListArg::builder()
                .hashes(hashes.join("|"))
                .build(),
            |torrents| -> bool {
                torrents.iter().flat_map(|t| t.save_path.as_ref()).any(|p| {
                    path_equals_as_file_url(p, new_path)
                        .inspect_err(|error| {
                            tracing::warn!(name = "path_equals_as_file_url", error = ?error);
                        })
                        .unwrap_or(false)
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    async fn get_torrent_path(&self, hashes: String) -> eyre::Result<Option<String>> {
        let mut torrent_list = self
            .client
            .get_torrent_list(GetTorrentListArg {
                hashes: Some(hashes),
                ..Default::default()
            })
            .await?;
        let torrent = torrent_list.first_mut().ok_or_eyre("No torrent found")?;
        Ok(torrent.save_path.take())
    }

    #[instrument(level = "debug", skip(self))]
    async fn check_connection(&self) -> eyre::Result<()> {
        self.api_version().await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    async fn set_torrents_category(&self, hashes: Vec<String>, category: &str) -> eyre::Result<()> {
        let result = self
            .client
            .set_torrent_category(hashes.clone(), category)
            .await;
        if let Err(qbit_rs::Error::ApiError(qbit_rs::ApiError::CategoryNotFound)) = &result {
            self.add_category(category).await?;
            self.client
                .set_torrent_category(hashes.clone(), category)
                .await?;
        } else {
            result?;
        }
        self.wait_torrents_until(
            GetTorrentListArg::builder()
                .hashes(hashes.join("|"))
                .build(),
            |torrents| {
                torrents
                    .iter()
                    .all(|t| t.category.as_ref().is_some_and(|c| c == category))
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    async fn add_torrent_tags(&self, hashes: Vec<String>, tags: Vec<String>) -> eyre::Result<()> {
        if tags.is_empty() {
            return Err(eyre::eyre!("add torrent tags can not be empty"));
        }
        self.client
            .add_torrent_tags(hashes.clone(), tags.clone())
            .await?;
        let tag_sets = tags.iter().map(|s| s.as_str()).collect::<HashSet<&str>>();
        self.wait_torrents_until(
            GetTorrentListArg::builder()
                .hashes(hashes.join("|"))
                .build(),
            |torrents| {
                torrents.iter().all(|t| {
                    t.tags.as_ref().is_some_and(|t| {
                        t.split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect::<HashSet<&str>>()
                            .is_superset(&tag_sets)
                    })
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    async fn add_category(&self, category: &str) -> eyre::Result<()> {
        self.client
            .add_category(
                NonEmptyStr::new(category).ok_or_eyre("category can not be empty")?,
                self.save_path.as_str(),
            )
            .await?;
        self.wait_sync_until(
            |sync_data| {
                sync_data
                    .categories
                    .as_ref()
                    .is_some_and(|s| s.contains_key(category))
            },
            None,
        )
        .await?;

        Ok(())
    }

    fn get_save_path(&self, sub_path: &Path) -> PathBuf {
        self.save_path.join(sub_path)
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
    use itertools::Itertools;

    use super::*;

    fn get_tmp_qbit_test_folder() -> &'static str {
        if cfg!(all(windows, not(feature = "testcontainers"))) {
            "C:\\Windows\\Temp\\konobangu\\qbit"
        } else {
            "/tmp/konobangu/qbit"
        }
    }

    #[cfg(feature = "testcontainers")]
    pub async fn create_qbit_testcontainer(
    ) -> eyre::Result<testcontainers::ContainerRequest<testcontainers::GenericImage>> {
        use testcontainers::{
            core::{
                ContainerPort,
                // ReuseDirective,
                WaitFor,
            },
            GenericImage,
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
        test_qbittorrent_downloader_impl(None, None).await;
    }

    #[cfg(feature = "testcontainers")]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_qbittorrent_downloader() -> eyre::Result<()> {
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
    ) -> eyre::Result<()> {
        let base_save_path = Path::new(get_tmp_qbit_test_folder());

        let mut downloader = QBittorrentDownloader::from_creation(QBittorrentDownloaderCreation {
            endpoint: "http://127.0.0.1:8080".to_string(),
            password: password.unwrap_or_default().to_string(),
            username: username.unwrap_or_default().to_string(),
            subscriber_id: 0,
            save_path: base_save_path.to_string(),
        })
        .await?;

        downloader.wait_sync_timeout = Duration::from_secs(3);

        downloader.check_connection().await?;

        downloader
            .delete_torrents(vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()])
            .await?;

        let torrent_source = TorrentSource::parse(
            None,
          "https://mikanani.me/Download/20240301/47ee2d69e7f19af783ad896541a07b012676f858.torrent"
      ).await?;

        let save_path = base_save_path.join(format!(
            "test_add_torrents_{}",
            chrono::Utc::now().timestamp()
        ));

        downloader
            .add_torrents(torrent_source, save_path.to_string(), Some("bangumi"))
            .await?;

        let get_torrent = async || -> eyre::Result<Torrent> {
            let torrent_infos = downloader
                .get_torrents_info(TorrentFilter::All, None, None)
                .await?;

            let result = torrent_infos
                .into_iter()
                .find(|t| (t.get_hash() == Some("47ee2d69e7f19af783ad896541a07b012676f858")))
                .ok_or_eyre("no torrent")?;

            Ok(result)
        };

        let target_torrent = get_torrent().await?;

        let files = target_torrent.iter_files().collect_vec();
        assert!(!files.is_empty());

        let first_file = files[0];
        assert_eq!(
            first_file.get_name(),
            r#"[Nekomoe kissaten&LoliHouse] Boku no Kokoro no Yabai Yatsu - 20 [WebRip 1080p HEVC-10bit AAC ASSx2].mkv"#
        );

        let test_tag = format!("test_tag_{}", chrono::Utc::now().timestamp());

        downloader
            .add_torrent_tags(
                vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()],
                vec![test_tag.clone()],
            )
            .await?;

        let target_torrent = get_torrent().await?;

        assert!(target_torrent.get_tags().iter().any(|s| s == &test_tag));

        let test_category = format!("test_category_{}", chrono::Utc::now().timestamp());

        downloader
            .set_torrents_category(
                vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()],
                &test_category,
            )
            .await?;

        let target_torrent = get_torrent().await?;

        assert_eq!(Some(test_category.as_str()), target_torrent.get_category());

        let moved_save_path = base_save_path.join(format!(
            "moved_test_add_torrents_{}",
            chrono::Utc::now().timestamp()
        ));

        downloader
            .move_torrents(
                vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()],
                moved_save_path.as_str(),
            )
            .await?;

        let target_torrent = get_torrent().await?;

        let content_path = target_torrent.iter_files().next().unwrap().get_name();

        let new_content_path = &format!("new_{}", content_path);

        downloader
            .rename_torrent_file(
                "47ee2d69e7f19af783ad896541a07b012676f858",
                content_path,
                new_content_path,
            )
            .await?;

        let target_torrent = get_torrent().await?;

        let content_path = target_torrent.iter_files().next().unwrap().get_name();

        assert_eq!(content_path, new_content_path);

        downloader
            .delete_torrents(vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()])
            .await?;

        let torrent_infos1 = downloader
            .get_torrents_info(TorrentFilter::All, None, None)
            .await?;

        assert!(torrent_infos1.is_empty());

        Ok(())
    }
}
