use std::borrow::Cow;

use bytes::Bytes;
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
pub struct MikanSubscriberRssChannel {
    pub mikan_subscription_token: String,
    pub url: Url,
    pub items: Vec<MikanRssItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MikanRssChannel {
    Bangumi(MikanBangumiRssChannel),
    Subscriber(MikanSubscriberRssChannel),
}

impl MikanRssChannel {
    pub fn items(&self) -> &[MikanRssItem] {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { items, .. })
            | Self::Subscriber(MikanSubscriberRssChannel { items, .. }) => items,
        }
    }

    pub fn into_items(self) -> Vec<MikanRssItem> {
        match self {
            Self::Bangumi(MikanBangumiRssChannel { items, .. })
            | Self::Subscriber(MikanSubscriberRssChannel { items, .. }) => items,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { name, .. }) => Some(name.as_str()),
            Self::Subscriber(MikanSubscriberRssChannel { .. }) => None,
        }
    }

    pub fn url(&self) -> &Url {
        match &self {
            Self::Bangumi(MikanBangumiRssChannel { url, .. })
            | Self::Subscriber(MikanSubscriberRssChannel { url, .. }) => url,
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
    pub mikan_fansub_id: String,
}

impl MikanBangumiRssUrlMeta {
    pub fn from_url(url: &Url) -> Option<Self> {
        if url.path() == "/RSS/Bangumi" {
            if let (Some(mikan_fansub_id), Some(mikan_bangumi_id)) = (
                url.query_pairs()
                    .find(|(k, _)| k == "subgroupid")
                    .map(|(_, v)| v.to_string()),
                url.query_pairs()
                    .find(|(k, _)| k == "bangumiId")
                    .map(|(_, v)| v.to_string()),
            ) {
                Some(MikanBangumiRssUrlMeta {
                    mikan_bangumi_id,
                    mikan_fansub_id,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSubscriberSubscriptionRssUrlMeta {
    pub mikan_subscription_token: String,
}

impl MikanSubscriberSubscriptionRssUrlMeta {
    pub fn from_url(url: &Url) -> Option<Self> {
        if url.path() == "/RSS/MyBangumi" {
            url.query_pairs().find(|(k, _)| k == "token").map(|(_, v)| {
                MikanSubscriberSubscriptionRssUrlMeta {
                    mikan_subscription_token: v.to_string(),
                }
            })
        } else {
            None
        }
    }
}

pub fn build_mikan_bangumi_subscription_rss_url(
    mikan_base_url: Url,
    mikan_bangumi_id: &str,
    mikan_fansub_id: Option<&str>,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path("/RSS/Bangumi");
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id);
    if let Some(mikan_fansub_id) = mikan_fansub_id {
        url.query_pairs_mut()
            .append_pair("subgroupid", mikan_fansub_id);
    };
    url
}

pub fn build_mikan_subscriber_subscription_rss_url(
    mikan_base_url: Url,
    mikan_subscription_token: &str,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path("/RSS/MyBangumi");
    url.query_pairs_mut()
        .append_pair("token", mikan_subscription_token);
    url
}
