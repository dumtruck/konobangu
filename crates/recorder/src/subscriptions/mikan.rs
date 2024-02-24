use crate::downloader::bytes::download_bytes;
use crate::downloader::defs::BITTORRENT_MIME_TYPE;

#[derive(Debug, Clone)]
pub struct MikanSubscriptionItem {
    pub item: rss::Item,
}

impl From<rss::Item> for MikanSubscriptionItem {
    fn from(item: rss::Item) -> Self {
        MikanSubscriptionItem {
            item
        }
    }
}

impl MikanSubscriptionItem {
    pub fn title(&self) -> &str {
        self.item.title().unwrap_or_default()
    }

    pub fn homepage(&self) -> Option<&str> {
        self.item.link()
    }

    pub fn torrent_url (&self) -> Option<&str> {
        self.item.enclosure().and_then(|en| {
            if en.mime_type == BITTORRENT_MIME_TYPE {
                Some(en.url.as_str())
            } else {
                None
            }
        })
    }
}

pub struct MikanSubscriptionEngine;

impl MikanSubscriptionEngine {
    pub async fn subscription_items_from_rss_url (
        url: &str
    ) -> eyre::Result<impl Iterator<Item = MikanSubscriptionItem>> {
        let bytes = download_bytes(url).await?;

        let channel = rss::Channel::read_from(&bytes[..])?;

        Ok(channel.items.into_iter().map(MikanSubscriptionItem::from))
    }
}
