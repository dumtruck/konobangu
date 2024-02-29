use std::fmt::Debug;

use eyre::OptionExt;
use futures::future::try_join_all;
use qbit_rs::{
    model::{AddTorrentArg, Credential, GetTorrentListArg, NonEmptyStr},
    Qbit,
};
use url::Url;

use super::{
    defs::{Torrent, TorrentFilter, TorrentSources},
    error::DownloaderError,
    torrent_downloader::TorrentDownloader,
};
use crate::{
    models::{entities::downloaders, prelude::DownloaderCategory},
    path::{VFSPathBuf, VFSSubPath},
};

pub struct QBittorrentDownloader {
    pub subscriber_id: i32,
    pub endpoint_url: Url,
    pub client: Qbit,
    pub save_path: String,
}

impl QBittorrentDownloader {
    pub fn from_downloader_model(model: downloaders::Model) -> Result<Self, DownloaderError> {
        if model.category != DownloaderCategory::QBittorrent {
            return Err(DownloaderError::InvalidMime {
                expected: DownloaderCategory::QBittorrent.to_string(),
                found: model.category.to_string(),
            });
        }

        let endpoint_url = model
            .endpoint_url()
            .map_err(DownloaderError::InvalidUrlFormat)?;
        let credential = Credential::new(model.username, model.password);
        let client = Qbit::new(endpoint_url.clone(), credential);

        Ok(Self {
            client,
            endpoint_url,
            subscriber_id: model.subscriber_id,
            save_path: model.download_path,
        })
    }

    async fn api_version(&self) -> eyre::Result<String> {
        let result = self.client.get_webapi_version().await?;
        Ok(result)
    }
}

#[async_trait::async_trait]
impl TorrentDownloader for QBittorrentDownloader {
    async fn get_torrents_info(
        &self,
        status_filter: TorrentFilter,
        category: String,
        tag: Option<String>,
    ) -> eyre::Result<Vec<Torrent>> {
        let arg = GetTorrentListArg {
            filter: Some(status_filter.into()),
            category: Some(category),
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
        source: TorrentSources,
        save_path: String,
        category: Option<String>,
    ) -> eyre::Result<()> {
        let arg = AddTorrentArg {
            source: source.into(),
            savepath: Some(save_path),
            category,
            auto_torrent_management: Some(false),
            ..Default::default()
        };
        self.client.add_torrent(arg).await?;
        Ok(())
    }

    async fn delete_torrents(&self, hashes: Vec<String>) -> eyre::Result<()> {
        self.client.delete_torrents(hashes, None).await?;
        Ok(())
    }

    async fn rename_torrent_file(
        &self,
        hash: &str,
        old_path: &str,
        new_path: &str,
    ) -> eyre::Result<()> {
        self.client.rename_file(hash, old_path, new_path).await?;
        Ok(())
    }

    async fn move_torrents(&self, hashes: Vec<String>, new_path: &str) -> eyre::Result<()> {
        self.client.set_torrent_location(hashes, new_path).await?;
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
        if category.is_empty() {
            return Err(eyre::anyhow!("Category cannot be empty"));
        }
        let result = self
            .client
            .set_torrent_category(hashes.clone(), category)
            .await;
        if let Err(qbit_rs::Error::ApiError(qbit_rs::ApiError::CategoryNotFound)) = result {
            self.client
                .add_category(
                    NonEmptyStr::new(category)
                        .unwrap_or_else(|| unreachable!("Category cannot be empty")),
                    self.save_path.as_str(),
                )
                .await?;
            self.client.set_torrent_category(hashes, category).await?;
        } else {
            result?;
        }
        Ok(())
    }

    async fn add_torrent_tags(&self, hashes: Vec<String>, tags: Vec<String>) -> eyre::Result<()> {
        self.client.add_torrent_tags(hashes, tags).await?;
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
