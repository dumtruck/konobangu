use downloaders::DownloaderCategory;
use quirks_path::{Path, PathBuf};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, IntoActiveModel};
use url::Url;

use super::{
    defs::{Torrent, TorrentFilter, TorrentSource},
    qbitorrent::QBittorrentDownloader,
};
use crate::{
    models::{bangumi, downloaders, downloads},
    path::torrent_path::gen_bangumi_sub_path,
};

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
