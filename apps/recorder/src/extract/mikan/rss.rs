use std::borrow::Cow;

use chrono::DateTime;
use downloader::bittorrent::defs::BITTORRENT_MIME_TYPE;
use fetch::{FetchError, IntoUrl, bytes::fetch_bytes};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

use crate::{
    errors::app_error::{RecorderError, RecorderResult},
    extract::mikan::{MikanClient, MikanEpisodeHomepageUrlMeta},
};

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
pub struct MikanBangumiIndexRssChannel {
    pub name: String,
    pub url: Url,
    pub mikan_bangumi_id: String,
    pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSubscriberStreamRssChannel {
    pub mikan_aggregation_id: String,
    pub url: Url,
    pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MikanRssChannel {
    Bangumi(MikanBangumiRssChannel),
    BangumiIndex(MikanBangumiIndexRssChannel),
    SubscriberStream(MikanSubscriberStreamRssChannel),
}

impl MikanRssChannel {
    pub fn items(&self) -> &[MikanRssItem] {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { items, .. })
            | Self::BangumiIndex(MikanBangumiIndexRssChannel { items, .. })
            | Self::SubscriberStream(MikanSubscriberStreamRssChannel { items, .. }) => items,
        }
    }

    pub fn into_items(self) -> Vec<MikanRssItem> {
        match self {
            Self::Bangumi(MikanBangumiRssChannel { items, .. })
            | Self::BangumiIndex(MikanBangumiIndexRssChannel { items, .. })
            | Self::SubscriberStream(MikanSubscriberStreamRssChannel { items, .. }) => items,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { name, .. })
            | Self::BangumiIndex(MikanBangumiIndexRssChannel { name, .. }) => Some(name.as_str()),
            Self::SubscriberStream(MikanSubscriberStreamRssChannel { .. }) => None,
        }
    }

    pub fn url(&self) -> &Url {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { url, .. })
            | Self::BangumiIndex(MikanBangumiIndexRssChannel { url, .. })
            | Self::SubscriberStream(MikanSubscriberStreamRssChannel { url, .. }) => url,
        }
    }
}

impl TryFrom<rss::Item> for MikanRssItem {
    type Error = RecorderError;

    fn try_from(item: rss::Item) -> Result<Self, Self::Error> {
        let enclosure = item.enclosure.ok_or_else(|| {
            RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("enclosure"))
        })?;

        let mime_type = enclosure.mime_type;
        if mime_type != BITTORRENT_MIME_TYPE {
            return Err(RecorderError::MimeError {
                expected: String::from(BITTORRENT_MIME_TYPE),
                found: mime_type.to_string(),
                desc: String::from("MikanRssItem"),
            });
        }

        let title = item.title.ok_or_else(|| {
            RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("title:title"))
        })?;

        let enclosure_url = Url::parse(&enclosure.url).map_err(|err| {
            RecorderError::from_mikan_rss_invalid_field_and_source(
                "enclosure_url:enclosure.link".into(),
                err,
            )
        })?;

        let homepage = item
            .link
            .and_then(|link| Url::parse(&link).ok())
            .ok_or_else(|| {
                RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("homepage:link"))
            })?;

        let MikanEpisodeHomepageUrlMeta {
            mikan_episode_id, ..
        } = MikanEpisodeHomepageUrlMeta::parse_url(&homepage).ok_or_else(|| {
            RecorderError::from_mikan_rss_invalid_field(Cow::Borrowed("mikan_episode_id"))
        })?;

        Ok(MikanRssItem {
            title,
            homepage,
            url: enclosure_url,
            content_length: enclosure.length.parse().ok(),
            mime: mime_type,
            pub_date: item
                .pub_date
                .and_then(|s| DateTime::parse_from_rfc2822(&s).ok())
                .map(|s| s.timestamp_millis()),
            mikan_episode_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MikanBangumiRssUrlMeta {
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MikanSubscriberAggregationRssUrlMeta {
    pub mikan_aggregation_id: String,
}

pub fn build_mikan_bangumi_rss_url(
    mikan_base_url: impl IntoUrl,
    mikan_bangumi_id: &str,
    mikan_fansub_id: Option<&str>,
) -> RecorderResult<Url> {
    let mut url = mikan_base_url.into_url().map_err(FetchError::from)?;
    url.set_path("/RSS/Bangumi");
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id);
    if let Some(mikan_fansub_id) = mikan_fansub_id {
        url.query_pairs_mut()
            .append_pair("subgroupid", mikan_fansub_id);
    };
    Ok(url)
}

pub fn build_mikan_subscriber_aggregation_rss_url(
    mikan_base_url: &str,
    mikan_aggregation_id: &str,
) -> RecorderResult<Url> {
    let mut url = Url::parse(mikan_base_url)?;
    url.set_path("/RSS/MyBangumi");
    url.query_pairs_mut()
        .append_pair("token", mikan_aggregation_id);
    Ok(url)
}

pub fn extract_mikan_bangumi_id_from_rss_url(url: &Url) -> Option<MikanBangumiRssUrlMeta> {
    if url.path() == "/RSS/Bangumi" {
        url.query_pairs()
            .find(|(k, _)| k == "bangumiId")
            .map(|(_, v)| MikanBangumiRssUrlMeta {
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

pub fn extract_mikan_subscriber_aggregation_id_from_rss_link(
    url: &Url,
) -> Option<MikanSubscriberAggregationRssUrlMeta> {
    if url.path() == "/RSS/MyBangumi" {
        url.query_pairs().find(|(k, _)| k == "token").map(|(_, v)| {
            MikanSubscriberAggregationRssUrlMeta {
                mikan_aggregation_id: v.to_string(),
            }
        })
    } else {
        None
    }
}

#[instrument(skip_all, fields(channel_rss_link = channel_rss_link.as_str()))]
pub async fn extract_mikan_rss_channel_from_rss_link(
    http_client: &MikanClient,
    channel_rss_link: impl IntoUrl,
) -> RecorderResult<MikanRssChannel> {
    let bytes = fetch_bytes(http_client, channel_rss_link.as_str()).await?;

    let channel = rss::Channel::read_from(&bytes[..])?;

    let channel_link = Url::parse(channel.link())?;

    if let Some(MikanBangumiRssUrlMeta {
        mikan_bangumi_id,
        mikan_fansub_id,
    }) = extract_mikan_bangumi_id_from_rss_url(&channel_link)
    {
        tracing::trace!(
            mikan_bangumi_id,
            mikan_fansub_id,
            "MikanBangumiRssLink extracting..."
        );

        let channel_name = channel.title().replace("Mikan Project - ", "");

        let items = channel
            .items
            .into_iter()
            .enumerate()
            .flat_map(|(idx, item)| {
                MikanRssItem::try_from(item).inspect_err(
                    |error| tracing::warn!(error = %error, "failed to extract rss item idx = {}", idx),
                )
            })
            .collect_vec();

        if let Some(mikan_fansub_id) = mikan_fansub_id {
            tracing::trace!(
                channel_name,
                channel_link = channel_link.as_str(),
                mikan_bangumi_id,
                mikan_fansub_id,
                "MikanBangumiRssChannel extracted"
            );

            Ok(MikanRssChannel::Bangumi(MikanBangumiRssChannel {
                name: channel_name,
                mikan_bangumi_id,
                mikan_fansub_id,
                url: channel_link,
                items,
            }))
        } else {
            tracing::trace!(
                channel_name,
                channel_link = channel_link.as_str(),
                mikan_bangumi_id,
                "MikanBangumiIndexRssChannel extracted"
            );

            Ok(MikanRssChannel::BangumiIndex(MikanBangumiIndexRssChannel {
                name: channel_name,
                mikan_bangumi_id,
                url: channel_link,
                items,
            }))
        }
    } else if let Some(MikanSubscriberAggregationRssUrlMeta {
        mikan_aggregation_id,
        ..
    }) = extract_mikan_subscriber_aggregation_id_from_rss_link(&channel_link)
    {
        tracing::trace!(
            mikan_aggregation_id,
            "MikanSubscriberAggregationRssLink extracting..."
        );

        let items = channel
            .items
            .into_iter()
            .enumerate()
            .flat_map(|(idx, item)| {
                MikanRssItem::try_from(item).inspect_err(
                |error| tracing::warn!(error = %error, "failed to extract rss item idx = {}", idx),
            )
            })
            .collect_vec();

        tracing::trace!(
            channel_link = channel_link.as_str(),
            mikan_aggregation_id,
            "MikanSubscriberAggregationRssChannel extracted"
        );

        Ok(MikanRssChannel::SubscriberStream(
            MikanSubscriberStreamRssChannel {
                mikan_aggregation_id,
                items,
                url: channel_link,
            },
        ))
    } else {
        Err(RecorderError::MikanRssInvalidFormatError).inspect_err(|error| {
            tracing::warn!(error = %error);
        })
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use downloader::bittorrent::BITTORRENT_MIME_TYPE;
    use rstest::rstest;
    use url::Url;

    use crate::{
        errors::RecorderResult,
        extract::mikan::{
            MikanBangumiIndexRssChannel, MikanBangumiRssChannel, MikanRssChannel,
            extract_mikan_rss_channel_from_rss_link,
        },
        test_utils::mikan::build_testing_mikan_client,
    };

    #[rstest]
    #[tokio::test]
    async fn test_parse_mikan_rss_channel_from_rss_link() -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;

        let mikan_base_url = Url::parse(&mikan_server.url())?;

        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        {
            let bangumi_rss_url =
                mikan_base_url.join("/RSS/Bangumi?bangumiId=3141&subgroupid=370")?;
            let bangumi_rss_mock = mikan_server
                .mock("GET", bangumi_rss_url.path())
                .with_body_from_file("tests/resources/mikan/Bangumi-3141-370.rss")
                .match_query(mockito::Matcher::Any)
                .create_async()
                .await;

            let channel = extract_mikan_rss_channel_from_rss_link(&mikan_client, bangumi_rss_url)
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

            assert!(
                &first_sub_item
                    .homepage
                    .as_str()
                    .starts_with("https://mikanani.me/Home/Episode")
            );

            let name = first_sub_item.title.as_str();
            assert!(name.contains("葬送的芙莉莲"));

            bangumi_rss_mock.expect(1);
        }
        {
            let bangumi_rss_url = mikan_base_url.join("/RSS/Bangumi?bangumiId=3416")?;

            let bangumi_rss_mock = mikan_server
                .mock("GET", bangumi_rss_url.path())
                .match_query(mockito::Matcher::Any)
                .with_body_from_file("tests/resources/mikan/Bangumi-3416.rss")
                .create_async()
                .await;

            let channel = extract_mikan_rss_channel_from_rss_link(&mikan_client, bangumi_rss_url)
                .await
                .expect("should get mikan channel from rss url");

            assert_matches!(
                &channel,
                MikanRssChannel::BangumiIndex(MikanBangumiIndexRssChannel { .. })
            );

            assert_matches!(&channel.name(), Some("叹气的亡灵想隐退"));

            bangumi_rss_mock.expect(1);
        }
        Ok(())
    }
}
