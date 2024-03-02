use chrono::DateTime;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};

use crate::{
    downloaders::{bytes::download_bytes, defs::BITTORRENT_MIME_TYPE},
    parsers::errors::ParseError,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanRssItem {
    pub title: String,
    pub homepage: Option<String>,
    pub url: String,
    pub content_length: Option<u64>,
    pub mime: String,
    pub pub_date: Option<i64>,
}

impl TryFrom<rss::Item> for MikanRssItem {
    type Error = ParseError;

    fn try_from(item: rss::Item) -> Result<Self, Self::Error> {
        let mime_type = item
            .enclosure()
            .map(|x| x.mime_type.to_string())
            .unwrap_or_default();
        if mime_type == BITTORRENT_MIME_TYPE {
            let enclosure = item.enclosure.unwrap();

            Ok(MikanRssItem {
                title: item.title.unwrap_or_default(),
                homepage: item.link,
                url: enclosure.url,
                content_length: enclosure.length.parse().ok(),
                mime: enclosure.mime_type,
                pub_date: item
                    .pub_date
                    .and_then(|s| DateTime::parse_from_rfc2822(&s).ok())
                    .map(|s| s.timestamp_millis()),
            })
        } else {
            Err(ParseError::MimeError {
                expected: String::from(BITTORRENT_MIME_TYPE),
                found: mime_type,
                desc: String::from("MikanRssItem"),
            })
        }
    }
}

pub async fn parse_mikan_rss_items_from_rss_link(
    url: impl IntoUrl,
) -> eyre::Result<impl Iterator<Item = MikanRssItem>> {
    let bytes = download_bytes(url).await?;

    let channel = rss::Channel::read_from(&bytes[..])?;

    Ok(channel.items.into_iter().flat_map(MikanRssItem::try_from))
}

#[cfg(test)]
mod tests {
    use super::parse_mikan_rss_items_from_rss_link;
    use crate::downloaders::defs::BITTORRENT_MIME_TYPE;

    #[tokio::test]
    pub async fn test_mikan_subscription_items_from_rss_url() {
        let url = "https://mikanani.me/RSS/Bangumi?bangumiId=3141&subgroupid=370";
        let items = parse_mikan_rss_items_from_rss_link(url)
            .await
            .expect("should get subscription items from rss url")
            .collect::<Vec<_>>();

        let first_sub_item = items
            .first()
            .expect("mikan subscriptions should have at least one subs");

        assert_eq!(first_sub_item.mime, BITTORRENT_MIME_TYPE);
        let homepage = first_sub_item
            .homepage
            .as_ref()
            .expect("mikan subscription item should have home page");
        assert!(homepage.starts_with("https://mikanani.me/Home/Episode"));
        let name = first_sub_item.title.as_str();
        assert!(name.contains("葬送的芙莉莲"));
    }
}
