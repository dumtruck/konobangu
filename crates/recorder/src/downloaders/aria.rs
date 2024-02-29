#![allow(unused_variables)]
use super::{
    defs::{Torrent, TorrentFilter, TorrentSources},
    torrent_downloader::TorrentDownloader,
};
use crate::path::{VFSPathBuf, VFSSubPath};

#[derive(Debug)]
pub struct AriaDownloader {}

#[async_trait::async_trait]
impl TorrentDownloader for AriaDownloader {
    async fn get_torrents_info(
        &self,
        status_filter: TorrentFilter,
        category: String,
        tag: Option<String>,
    ) -> eyre::Result<Vec<Torrent>> {
        unimplemented!()
    }

    async fn add_torrents(
        &self,
        source: TorrentSources,
        save_path: String,
        category: Option<String>,
    ) -> eyre::Result<()> {
        unimplemented!()
    }

    async fn delete_torrents(&self, hashes: Vec<String>) -> eyre::Result<()> {
        unimplemented!()
    }

    async fn rename_torrent_file(
        &self,
        hash: &str,
        old_path: &str,
        new_path: &str,
    ) -> eyre::Result<()> {
        unimplemented!()
    }

    async fn move_torrents(&self, hashes: Vec<String>, new_path: &str) -> eyre::Result<()> {
        unimplemented!()
    }

    async fn get_torrent_path(&self, hashes: String) -> eyre::Result<Option<String>> {
        unimplemented!()
    }

    async fn check_connection(&self) -> eyre::Result<()> {
        unimplemented!()
    }

    async fn set_torrents_category(&self, hashes: Vec<String>, category: &str) -> eyre::Result<()> {
        unimplemented!()
    }

    async fn add_torrent_tags(&self, hashes: Vec<String>, tags: Vec<String>) -> eyre::Result<()> {
        unimplemented!()
    }

    fn get_save_path(&self, sub_path: &VFSSubPath) -> VFSPathBuf {
        unimplemented!()
    }
}
