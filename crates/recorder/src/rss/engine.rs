use serde::{Deserialize, Serialize};

use crate::models::subscriptions::subscriptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssTorrent {}

#[derive(Debug)]
pub struct RssEngine {}

impl RssEngine {
    // pub async fn get_rss_torrents(
    //     rss_subscription: &subscriptions::ActiveModel,
    // ) -> eyre::Result<Vec<RssTorrent>> {
    //     Ok(())
    // }

    pub async fn get_torrents(url: &str) -> eyre::Result<rss::Channel> {
        let content = reqwest::get(url).await?.bytes().await?;
        let channel: rss::Channel = rss::Channel::read_from(&content[..])?;
        Ok(channel)
    }
}
