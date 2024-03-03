use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    future::Future,
    sync::Arc,
    time::Duration,
};

use eyre::OptionExt;
use futures::future::try_join_all;
use qbit_rs::{
    model::{AddTorrentArg, Credential, GetTorrentListArg, NonEmptyStr, SyncData},
    Qbit,
};
use tokio::time::sleep;
use url::Url;

use super::{
    defs::{Torrent, TorrentFilter, TorrentSource},
    error::DownloaderError,
    torrent_downloader::TorrentDownloader,
};
use crate::{
    downloaders::defs::{QbitTorrent, QbitTorrentContent, TorrentContent},
    models::{entities::downloaders, prelude::DownloaderCategory},
    path::{path_str_equals, VFSPathBuf, VFSSubPath},
};

pub struct SyncDataCache {
    pub torrents: HashMap<String, QbitTorrent>,
}

pub struct QBittorrentDownloader {
    pub subscriber_id: i32,
    pub endpoint_url: Url,
    pub client: Arc<Qbit>,
    pub save_path: String,
    pub wait_sync_timeout: Duration,
}

impl QBittorrentDownloader {
    pub async fn from_downloader_model(model: downloaders::Model) -> Result<Self, DownloaderError> {
        if model.category != DownloaderCategory::QBittorrent {
            return Err(DownloaderError::InvalidMime {
                expected: DownloaderCategory::QBittorrent.to_string(),
                found: model.category.to_string(),
            });
        }

        let endpoint_url = model
            .endpoint_url()
            .map_err(DownloaderError::InvalidUrlParse)?;
        let credential = Credential::new(model.username, model.password);
        let client = Qbit::new(endpoint_url.clone(), credential);

        client
            .login(false)
            .await
            .map_err(DownloaderError::QBitAPIError)?;

        client.sync(None).await?.rid;

        Ok(Self {
            client: Arc::new(client),
            endpoint_url,
            subscriber_id: model.subscriber_id,
            save_path: model.save_path,
            wait_sync_timeout: Duration::from_millis(10000),
        })
    }

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
        F: FnMut(D) -> bool,
        E: Clone,
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
                if stop_wait_fn(sync_data) {
                    break;
                } else {
                    return Err(DownloaderError::TimeoutError {
                        action: Cow::Borrowed("QBittorrentDownloader::wait_unit"),
                        timeout,
                    }
                    .into());
                }
            }
            let sync_data = fetch_data_fn(self.client.clone(), env.clone()).await?;
            if stop_wait_fn(sync_data) {
                break;
            }
            next_wait_ms *= 2;
        }
        Ok(())
    }

    pub async fn wait_torrents_until<F>(
        &self,
        arg: GetTorrentListArg,
        stop_wait_fn: F,
        timeout: Option<Duration>,
    ) -> eyre::Result<()>
    where
        F: FnMut(Vec<QbitTorrent>) -> bool,
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

    pub async fn wait_sync_until<F: FnMut(SyncData) -> bool>(
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

    async fn wait_torrent_contents_until<F: FnMut(Vec<QbitTorrentContent>) -> bool>(
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

#[async_trait::async_trait]
impl TorrentDownloader for QBittorrentDownloader {
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
                    .map_or(false, |t| t.contains_key(source_hash))
            },
            None,
        )
        .await?;
        Ok(())
    }

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

    async fn rename_torrent_file(
        &self,
        hash: &str,
        old_path: &str,
        new_path: &str,
    ) -> eyre::Result<()> {
        self.client.rename_file(hash, old_path, new_path).await?;
        self.wait_torrent_contents_until(
            hash,
            |contents| -> bool {
                contents
                    .iter()
                    .any(|c| path_str_equals(c.get_name(), new_path).unwrap_or(false))
            },
            None,
        )
        .await?;
        Ok(())
    }

    async fn move_torrents(&self, hashes: Vec<String>, new_path: &str) -> eyre::Result<()> {
        self.client
            .set_torrent_location(hashes.clone(), new_path)
            .await?;
        self.wait_torrents_until(
            GetTorrentListArg::builder()
                .hashes(hashes.join("|"))
                .build(),
            |torrents| -> bool {
                torrents.iter().all(|t| {
                    t.save_path
                        .as_ref()
                        .map_or(false, |p| path_str_equals(p, new_path).unwrap_or(false))
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

    async fn check_connection(&self) -> eyre::Result<()> {
        self.api_version().await?;
        Ok(())
    }

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
                    .all(|t| t.category.as_ref().map_or(false, |c| c == category))
            },
            None,
        )
        .await?;
        Ok(())
    }

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
                    t.tags.as_ref().map_or(false, |t| {
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
                    .map_or(false, |s| s.contains_key(category))
            },
            None,
        )
        .await?;

        Ok(())
    }

    fn get_save_path(&self, sub_path: &VFSSubPath) -> VFSPathBuf {
        VFSPathBuf::new(self.save_path.clone(), sub_path.to_path_buf())
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
    use testcontainers::core::ExecCommand;

    use super::*;

    fn get_tmp_qbit_test_folder() -> &'static str {
        if cfg!(windows) {
            "C:\\Windows\\Temp\\konobangu\\qbit"
        } else {
            "/tmp/konobangu/qbit"
        }
    }

    #[cfg(feature = "testcontainers")]
    pub fn create_qbit_testcontainer() -> testcontainers::RunnableImage<testcontainers::GenericImage>
    {
        let image = testcontainers::RunnableImage::from(
            testcontainers::GenericImage::new("linuxserver/qbittorrent", "latest")
                .with_wait_for(testcontainers::core::WaitFor::message_on_stderr(
                    "Connection to localhost",
                ))
                .with_env_var("WEBUI_PORT", "8080")
                .with_env_var("TZ", "Asia/Singapore")
                .with_env_var("TORRENTING_PORT", "6881")
                .with_exposed_port(8080)
                .with_exposed_port(6881),
        );

        image
    }

    #[cfg(not(feature = "testcontainers"))]
    #[tokio::test]
    async fn test_qbittorrent_downloader() {
        test_qbittorrent_downloader_impl().await;
    }

    // @TODO: not support now, testcontainers crate not support to read logs to get
    // password
    #[cfg(feature = "testcontainers")]
    #[tokio::test]
    async fn test_qbittorrent_downloader() {
        let docker = testcontainers::clients::Cli::default();
        let image = create_qbit_testcontainer();

        let container = docker.run(image);

        let mut exec = ExecCommand::default();

        container.exec(exec);

        test_qbittorrent_downloader_impl().await;
    }

    async fn test_qbittorrent_downloader_impl() {
        let base_save_path = VFSSubPath::new(get_tmp_qbit_test_folder());

        let downloader = QBittorrentDownloader::from_downloader_model(downloaders::Model {
            created_at: Default::default(),
            updated_at: Default::default(),
            id: 0,
            category: DownloaderCategory::QBittorrent,
            endpoint: "http://localhost:8080".to_string(),
            password: "".to_string(),
            username: "".to_string(),
            subscriber_id: 0,
            save_path: base_save_path.to_string(),
        })
        .await
        .unwrap();

        downloader.check_connection().await.unwrap();

        downloader
            .delete_torrents(vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()])
            .await
            .unwrap();

        let torrent_source = TorrentSource::parse(
            "https://mikanani.me/Download/20240301/47ee2d69e7f19af783ad896541a07b012676f858.torrent"
        ).await.unwrap();

        let save_path = base_save_path.join(format!(
            "test_add_torrents_{}",
            chrono::Utc::now().timestamp()
        ));

        downloader
            .add_torrents(torrent_source, save_path.to_string(), Some("bangumi"))
            .await
            .unwrap();

        let get_torrent = async || -> eyre::Result<Torrent> {
            let torrent_infos = downloader
                .get_torrents_info(TorrentFilter::All, None, None)
                .await?;

            let result = torrent_infos
                .into_iter()
                .find(|t| {
                    t.get_hash()
                        .map_or(false, |s| s == "47ee2d69e7f19af783ad896541a07b012676f858")
                })
                .ok_or_eyre("no torrent")?;

            Ok(result)
        };

        let target_torrent = get_torrent().await.unwrap();

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
            .await
            .unwrap();

        let target_torrent = get_torrent().await.unwrap();

        assert!(target_torrent.get_tags().iter().any(|s| s == &test_tag));

        let test_category = format!("test_category_{}", chrono::Utc::now().timestamp());

        downloader
            .set_torrents_category(
                vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()],
                &test_category,
            )
            .await
            .unwrap();

        let target_torrent = get_torrent().await.unwrap();

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
            .await
            .unwrap();

        let target_torrent = get_torrent().await.unwrap();

        let content_path = target_torrent.iter_files().next().unwrap().get_name();

        let new_content_path = &format!("new_{}", content_path);

        downloader
            .rename_torrent_file(
                "47ee2d69e7f19af783ad896541a07b012676f858",
                content_path,
                new_content_path,
            )
            .await
            .unwrap();

        let target_torrent = get_torrent().await.unwrap();

        let content_path = target_torrent.iter_files().next().unwrap().get_name();

        assert_eq!(content_path, new_content_path);

        downloader
            .delete_torrents(vec!["47ee2d69e7f19af783ad896541a07b012676f858".to_string()])
            .await
            .unwrap();

        let torrent_infos1 = downloader
            .get_torrents_info(TorrentFilter::All, None, None)
            .await
            .unwrap();

        assert!(torrent_infos1.is_empty());
    }
}
