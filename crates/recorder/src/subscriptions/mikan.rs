use crate::downloader::bytes::download_bytes;
use crate::downloader::defs::BITTORRENT_MIME_TYPE;

#[derive(Debug, Clone)]
pub struct MikanSubscriptionItem {
    pub title: String,
    pub home_page: Option<String>,
    pub url: String,
    pub content_length: Option<u64>,
    pub mime: String,
    pub pub_date: Option<String>,
}

impl MikanSubscriptionItem {
    pub fn from_rss_item(item: rss::Item) -> Option<Self> {
        let mime_match = item.enclosure()
            .map(|x| x.mime_type == BITTORRENT_MIME_TYPE)
            .unwrap_or_default();
        if mime_match {
            let enclosure = item.enclosure.unwrap();
            let content_length = enclosure.length.parse().ok();
            Some(MikanSubscriptionItem {
                title: item.title.unwrap_or_default(),
                home_page: item.link,
                url: enclosure.url,
                content_length,
                mime: enclosure.mime_type,
                pub_date: item.pub_date,
            })
        } else {
            None
        }
    }
}

pub struct MikanSubscriptionEngine;

impl MikanSubscriptionEngine {
    pub async fn subscription_items_from_rss_url(
        url: &str
    ) -> eyre::Result<impl Iterator<Item=MikanSubscriptionItem>> {
        let bytes = download_bytes(url).await?;

        let channel = rss::Channel::read_from(&bytes[..])?;

        Ok(channel.items.into_iter().flat_map(MikanSubscriptionItem::from_rss_item))
    }
}
