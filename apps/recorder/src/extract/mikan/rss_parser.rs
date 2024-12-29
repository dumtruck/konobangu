use std::ops::Deref;

use chrono::DateTime;
use itertools::Itertools;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use torrent::core::BITTORRENT_MIME_TYPE;
use url::Url;

use super::{
    web_parser::{parse_mikan_episode_id_from_homepage, MikanEpisodeHomepage},
    AppMikanClient,
};
use crate::{extract::errors::ParseError, fetch::bytes::download_bytes_with_client};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanRssItem {
    pub title: String,
    pub homepage: Url,
    pub url: Url,
    pub content_length: Option<u64>,
    pub mime: String,
    pub pub_date: Option<i64>,
    pub mikan_episode_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanBangumiRssChannel {
    pub name: String,
    pub url: Url,
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: String,
    pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanBangumiAggregationRssChannel {
    pub name: String,
    pub url: Url,
    pub mikan_bangumi_id: String,
    pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSubscriberAggregationRssChannel {
    pub mikan_aggregation_id: String,
    pub url: Url,
    pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MikanRssChannel {
    Bangumi(MikanBangumiRssChannel),
    BangumiAggregation(MikanBangumiAggregationRssChannel),
    SubscriberAggregation(MikanSubscriberAggregationRssChannel),
}

impl MikanRssChannel {
    pub fn items(&self) -> &[MikanRssItem] {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { items, .. })
            | Self::BangumiAggregation(MikanBangumiAggregationRssChannel { items, .. })
            | Self::SubscriberAggregation(MikanSubscriberAggregationRssChannel { items, .. }) => {
                items
            }
        }
    }

    pub fn into_items(self) -> Vec<MikanRssItem> {
        match self {
            Self::Bangumi(MikanBangumiRssChannel { items, .. })
            | Self::BangumiAggregation(MikanBangumiAggregationRssChannel { items, .. })
            | Self::SubscriberAggregation(MikanSubscriberAggregationRssChannel { items, .. }) => {
                items
            }
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { name, .. })
            | Self::BangumiAggregation(MikanBangumiAggregationRssChannel { name, .. }) => {
                Some(name.as_str())
            }
            Self::SubscriberAggregation(MikanSubscriberAggregationRssChannel { .. }) => None,
        }
    }

    pub fn url(&self) -> &Url {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { url, .. })
            | Self::BangumiAggregation(MikanBangumiAggregationRssChannel { url, .. })
            | Self::SubscriberAggregation(MikanSubscriberAggregationRssChannel { url, .. }) => url,
        }
    }
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

            let homepage = item
                .link
                .ok_or_else(|| ParseError::MikanRssItemFormatError {
                    reason: String::from("must to have link for homepage"),
                })?;

            let homepage = Url::parse(&homepage)?;

            let enclosure_url = Url::parse(&enclosure.url)?;

            let MikanEpisodeHomepage {
                mikan_episode_id, ..
            } = parse_mikan_episode_id_from_homepage(&homepage).ok_or_else(|| {
                ParseError::MikanRssItemFormatError {
                    reason: String::from("homepage link format invalid"),
                }
            })?;

            Ok(MikanRssItem {
                title: item.title.unwrap_or_default(),
                homepage,
                url: enclosure_url,
                content_length: enclosure.length.parse().ok(),
                mime: enclosure.mime_type,
                pub_date: item
                    .pub_date
                    .and_then(|s| DateTime::parse_from_rfc2822(&s).ok())
                    .map(|s| s.timestamp_millis()),
                mikan_episode_id,
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

#[derive(Debug, Clone)]
pub struct MikanBangumiRssLink {
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MikanSubscriberAggregationRssLink {
    pub mikan_aggregation_id: String,
}

pub fn build_mikan_bangumi_rss_link(
    mikan_base_url: &str,
    mikan_bangumi_id: &str,
    mikan_fansub_id: Option<&str>,
) -> eyre::Result<Url> {
    let mut url = Url::parse(mikan_base_url)?;
    url.set_path("/RSS/Bangumi");
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id);
    if let Some(mikan_fansub_id) = mikan_fansub_id {
        url.query_pairs_mut()
            .append_pair("subgroupid", mikan_fansub_id);
    };
    Ok(url)
}

pub fn build_mikan_subscriber_aggregation_rss_link(
    mikan_base_url: &str,
    mikan_aggregation_id: &str,
) -> eyre::Result<Url> {
    let mut url = Url::parse(mikan_base_url)?;
    url.set_path("/RSS/MyBangumi");
    url.query_pairs_mut()
        .append_pair("token", mikan_aggregation_id);
    Ok(url)
}

pub fn parse_mikan_bangumi_id_from_rss_link(url: &Url) -> Option<MikanBangumiRssLink> {
    if url.path() == "/RSS/Bangumi" {
        url.query_pairs()
            .find(|(k, _)| k == "bangumiId")
            .map(|(_, v)| MikanBangumiRssLink {
                mikan_bangumi_id: v.to_string(),
                mikan_fansub_id: url
                    .query_pairs()
                    .find(|(k, _)| k == "subgroupid")
                    .map(|(_, v)| v.to_string()),
            })
    } else {
        None
    }
}

pub fn parse_mikan_subscriber_aggregation_id_from_rss_link(
    url: &Url,
) -> Option<MikanSubscriberAggregationRssLink> {
    if url.path() == "/RSS/MyBangumi" {
        url.query_pairs().find(|(k, _)| k == "token").map(|(_, v)| {
            MikanSubscriberAggregationRssLink {
                mikan_aggregation_id: v.to_string(),
            }
        })
    } else {
        None
    }
}

pub async fn parse_mikan_rss_items_from_rss_link(
    client: Option<&AppMikanClient>,
    url: impl IntoUrl,
) -> eyre::Result<Vec<MikanRssItem>> {
    let channel = parse_mikan_rss_channel_from_rss_link(client, url).await?;

    Ok(channel.into_items())
}

pub async fn parse_mikan_rss_channel_from_rss_link(
    client: Option<&AppMikanClient>,
    url: impl IntoUrl,
) -> eyre::Result<MikanRssChannel> {
    let http_client = client.map(|s| s.deref());
    let bytes = download_bytes_with_client(http_client, url.as_str()).await?;

    let channel = rss::Channel::read_from(&bytes[..])?;

    let channel_link = Url::parse(channel.link())?;

    if let Some(MikanBangumiRssLink {
        mikan_bangumi_id,
        mikan_fansub_id,
    }) = parse_mikan_bangumi_id_from_rss_link(&channel_link)
    {
        let channel_name = channel.title().replace("Mikan Project - ", "");

        let items = channel
            .items
            .into_iter()
            // @TODO log error
            .flat_map(MikanRssItem::try_from)
            .collect_vec();

        if let Some(mikan_fansub_id) = mikan_fansub_id {
            Ok(MikanRssChannel::Bangumi(MikanBangumiRssChannel {
                name: channel_name,
                mikan_bangumi_id,
                mikan_fansub_id,
                url: channel_link,
                items,
            }))
        } else {
            Ok(MikanRssChannel::BangumiAggregation(
                MikanBangumiAggregationRssChannel {
                    name: channel_name,
                    mikan_bangumi_id,
                    url: channel_link,
                    items,
                },
            ))
        }
    } else if let Some(MikanSubscriberAggregationRssLink {
        mikan_aggregation_id,
        ..
    }) = parse_mikan_subscriber_aggregation_id_from_rss_link(&channel_link)
    {
        let items = channel
            .items
            .into_iter()
            // @TODO log error
            .flat_map(MikanRssItem::try_from)
            .collect_vec();

        return Ok(MikanRssChannel::SubscriberAggregation(
            MikanSubscriberAggregationRssChannel {
                mikan_aggregation_id,
                items,
                url: channel_link,
            },
        ));
    } else {
        return Err(ParseError::MikanRssFormatError {
            url: url.as_str().into(),
        }
        .into());
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use torrent::core::BITTORRENT_MIME_TYPE;

    use crate::extract::mikan::{
        parse_mikan_rss_channel_from_rss_link, MikanBangumiAggregationRssChannel,
        MikanBangumiRssChannel, MikanRssChannel,
    };

    #[tokio::test]
    pub async fn test_parse_mikan_rss_channel_from_rss_link() {
        {
            let bangumi_url = "https://mikanani.me/RSS/Bangumi?bangumiId=3141&subgroupid=370";

            let channel = parse_mikan_rss_channel_from_rss_link(None, bangumi_url)
                .await
                .expect("should get mikan channel from rss url");

            assert_matches!(
                &channel,
                MikanRssChannel::Bangumi(MikanBangumiRssChannel { .. })
            );

            assert_matches!(&channel.name(), Some("葬送的芙莉莲"));

            let items = channel.items();
            let first_sub_item = items
                .first()
                .expect("mikan subscriptions should have at least one subs");

            assert_eq!(first_sub_item.mime, BITTORRENT_MIME_TYPE);

            assert!(&first_sub_item
                .homepage
                .as_str()
                .starts_with("https://mikanani.me/Home/Episode"));

            let name = first_sub_item.title.as_str();
            assert!(name.contains("葬送的芙莉莲"));
        }
        {
            let bangumi_url = "https://mikanani.me/RSS/Bangumi?bangumiId=3416";

            let channel = parse_mikan_rss_channel_from_rss_link(None, bangumi_url)
                .await
                .expect("should get mikan channel from rss url");

            assert_matches!(
                &channel,
                MikanRssChannel::BangumiAggregation(MikanBangumiAggregationRssChannel { .. })
            );

            assert_matches!(&channel.name(), Some("叹气的亡灵想隐退"));
        }
    }
}
