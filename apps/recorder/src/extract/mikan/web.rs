use std::{borrow::Cow, fmt, sync::Arc};

use async_stream::try_stream;
use bytes::Bytes;
use fetch::{html::fetch_html, image::fetch_image};
use futures::{Stream, TryStreamExt, pin_mut};
use html_escape::decode_html_entities;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use snafu::OptionExt;
use tracing::instrument;
use url::Url;

use super::{
    MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH, MIKAN_POSTER_BUCKET_KEY,
    MIKAN_SEASON_FLOW_PAGE_PATH, MikanBangumiRssUrlMeta, MikanClient,
    extract_mikan_bangumi_id_from_rss_url,
};
use crate::{
    app::AppContextTrait,
    errors::app_error::{RecorderError, RecorderResult},
    extract::{
        html::{extract_background_image_src_from_style_attr, extract_inner_text_from_element_ref},
        media::extract_image_src_from_str,
    },
    storage::{StorageContentCategory, StorageServiceTrait},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct MikanBangumiIndexMeta {
    pub homepage: Url,
    pub origin_poster_src: Option<Url>,
    pub bangumi_title: String,
    pub mikan_bangumi_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct MikanFansubMeta {
    pub mikan_fansub_id: String,
    pub fansub: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct MikanBangumiMeta {
    pub homepage: Url,
    pub origin_poster_src: Option<Url>,
    pub bangumi_title: String,
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: String,
    pub fansub: String,
}

#[async_graphql::Object]
impl MikanBangumiMeta {
    async fn homepage(&self) -> &str {
        self.homepage.as_str()
    }

    async fn origin_poster_src(&self) -> Option<&str> {
        self.origin_poster_src.as_ref().map(|url| url.as_str())
    }

    async fn bangumi_title(&self) -> &str {
        &self.bangumi_title
    }

    async fn mikan_bangumi_id(&self) -> &str {
        &self.mikan_bangumi_id
    }

    async fn mikan_fansub_id(&self) -> &str {
        &self.mikan_fansub_id
    }

    async fn fansub(&self) -> &str {
        &self.fansub
    }
}

impl MikanBangumiMeta {
    pub fn from_bangumi_index_and_fansub_meta(
        bangumi_index_meta: MikanBangumiIndexMeta,
        fansub_meta: MikanFansubMeta,
    ) -> Self {
        Self {
            homepage: bangumi_index_meta.homepage,
            origin_poster_src: bangumi_index_meta.origin_poster_src,
            bangumi_title: bangumi_index_meta.bangumi_title,
            mikan_bangumi_id: bangumi_index_meta.mikan_bangumi_id,
            mikan_fansub_id: fansub_meta.mikan_fansub_id,
            fansub: fansub_meta.fansub,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpisodeMeta {
    pub homepage: Url,
    pub origin_poster_src: Option<Url>,
    pub bangumi_title: String,
    pub episode_title: String,
    pub fansub: String,
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: String,
    pub mikan_episode_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiPosterMeta {
    pub origin_poster_src: Url,
    pub poster_src: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiIndexHomepageUrlMeta {
    pub mikan_bangumi_id: String,
}

impl MikanBangumiIndexHomepageUrlMeta {
    pub fn parse_url(url: &Url) -> Option<Self> {
        if url.path().starts_with("/Home/Bangumi/") {
            let mikan_bangumi_id = url.path().replace("/Home/Bangumi/", "");

            Some(Self { mikan_bangumi_id })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiHomepageUrlMeta {
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: String,
}

impl MikanBangumiHomepageUrlMeta {
    pub fn from_url(url: &Url) -> Option<Self> {
        if url.path().starts_with("/Home/Bangumi/") {
            let mikan_bangumi_id = url.path().replace("/Home/Bangumi/", "");

            let url_fragment = url.fragment()?;

            Some(Self {
                mikan_bangumi_id,
                mikan_fansub_id: String::from(url_fragment),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpisodeHomepageUrlMeta {
    pub mikan_episode_id: String,
}

impl MikanEpisodeHomepageUrlMeta {
    pub fn parse_url(url: &Url) -> Option<Self> {
        if url.path().starts_with("/Home/Episode/") {
            let mikan_episode_id = url.path().replace("/Home/Episode/", "");
            Some(Self { mikan_episode_id })
        } else {
            None
        }
    }
}

#[derive(async_graphql::Enum, Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MikanSeasonStr {
    #[serde(rename = "春")]
    #[graphql(name = "spring")]
    Spring,
    #[serde(rename = "夏")]
    #[graphql(name = "summer")]
    Summer,
    #[serde(rename = "秋")]
    #[graphql(name = "autumn")]
    Autumn,
    #[serde(rename = "冬")]
    #[graphql(name = "winter")]
    Winter,
}

impl fmt::Display for MikanSeasonStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spring => write!(f, "春"),
            Self::Summer => write!(f, "夏"),
            Self::Autumn => write!(f, "秋"),
            Self::Winter => write!(f, "冬"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanSeasonFlowUrlMeta {
    pub year: i32,
    pub season_str: MikanSeasonStr,
}

pub fn build_mikan_bangumi_homepage_url(
    mikan_base_url: Url,
    mikan_bangumi_id: &str,
    mikan_fansub_id: Option<&str>,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path(&format!("/Home/Bangumi/{mikan_bangumi_id}"));
    url.set_fragment(mikan_fansub_id);
    url
}

pub fn build_mikan_season_flow_url(
    mikan_base_url: Url,
    year: i32,
    season_str: MikanSeasonStr,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path(MIKAN_SEASON_FLOW_PAGE_PATH);
    url.query_pairs_mut()
        .append_pair("year", &year.to_string())
        .append_pair("seasonStr", &season_str.to_string());
    url
}

pub fn build_mikan_episode_homepage_url(mikan_base_url: Url, mikan_episode_id: &str) -> Url {
    let mut url = mikan_base_url;
    url.set_path(&format!("/Home/Episode/{mikan_episode_id}"));
    url
}

pub fn build_mikan_bangumi_expand_subscribed_url(
    mikan_base_url: Url,
    mikan_bangumi_id: &str,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path(MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH);
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id)
        .append_pair("showSubscribed", "true");
    url
}

#[instrument(err, skip_all, fields(mikan_episode_homepage_url = mikan_episode_homepage_url.as_str()))]
pub fn extract_mikan_episode_meta_from_episode_homepage_html(
    html: &Html,
    mikan_base_url: Url,
    mikan_episode_homepage_url: Url,
) -> RecorderResult<MikanEpisodeMeta> {
    let bangumi_title_selector =
        &Selector::parse(".bangumi-title > a[href^='/Home/Bangumi/']").unwrap();
    let mikan_bangumi_id_selector =
        &Selector::parse(".bangumi-title > a.mikan-rss[data-original-title='RSS']").unwrap();
    let bangumi_poster_selector = &Selector::parse(".bangumi-poster").unwrap();

    let bangumi_title = html
        .select(bangumi_title_selector)
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title"))
        })?;

    let MikanBangumiRssUrlMeta {
        mikan_bangumi_id,
        mikan_fansub_id,
        ..
    } = html
        .select(mikan_bangumi_id_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(|s| mikan_episode_homepage_url.join(s).ok())
        .and_then(|rss_link_url| extract_mikan_bangumi_id_from_rss_url(&rss_link_url))
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_bangumi_id"))
        })?;

    let mikan_fansub_id = mikan_fansub_id.ok_or_else(|| {
        RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_fansub_id"))
    })?;

    let episode_title = html
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("episode_title"))
        })?;

    let MikanEpisodeHomepageUrlMeta {
        mikan_episode_id, ..
    } = MikanEpisodeHomepageUrlMeta::parse_url(&mikan_episode_homepage_url).ok_or_else(|| {
        RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_episode_id"))
    })?;

    let fansub_name = html
        .select(
            &Selector::parse(".bangumi-info a.magnet-link-wrap[href^='/Home/PublishGroup/']")
                .unwrap(),
        )
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("fansub_name"))
        })?;

    let origin_poster_src = html.select(bangumi_poster_selector).next().and_then(|el| {
        el.value()
            .attr("data-src")
            .and_then(|data_src| extract_image_src_from_str(data_src, &mikan_base_url))
            .or_else(|| {
                el.value().attr("style").and_then(|style| {
                    extract_background_image_src_from_style_attr(style, &mikan_base_url)
                })
            })
    });

    tracing::trace!(
        bangumi_title,
        mikan_bangumi_id,
        episode_title,
        mikan_episode_id,
        origin_poster_src = origin_poster_src.as_ref().map(|url| url.as_str()),
        fansub_name,
        mikan_fansub_id,
        "mikan episode meta extracted"
    );

    Ok(MikanEpisodeMeta {
        mikan_bangumi_id,
        mikan_fansub_id,
        bangumi_title,
        episode_title,
        homepage: mikan_episode_homepage_url,
        origin_poster_src,
        fansub: fansub_name,
        mikan_episode_id,
    })
}

#[instrument(skip_all, fields(mikan_episode_homepage_url = mikan_episode_homepage_url.as_str()))]
pub async fn scrape_mikan_episode_meta_from_episode_homepage_url(
    http_client: &MikanClient,
    mikan_episode_homepage_url: Url,
) -> RecorderResult<MikanEpisodeMeta> {
    let mikan_base_url = http_client.base_url().clone();
    let content = fetch_html(http_client, mikan_episode_homepage_url.as_str()).await?;

    let html = Html::parse_document(&content);

    extract_mikan_episode_meta_from_episode_homepage_html(
        &html,
        mikan_base_url,
        mikan_episode_homepage_url,
    )
}

pub fn extract_mikan_bangumi_index_meta_from_bangumi_homepage_html(
    html: &Html,
    mikan_bangumi_homepage_url: Url,
    mikan_base_url: &Url,
) -> RecorderResult<MikanBangumiIndexMeta> {
    let bangumi_title_selector = &Selector::parse(".bangumi-title").unwrap();
    let mikan_bangumi_id_selector =
        &Selector::parse(".bangumi-title > .mikan-rss[data-original-title='RSS']").unwrap();
    let bangumi_poster_selector = &Selector::parse(".bangumi-poster").unwrap();

    let bangumi_title = html
        .select(bangumi_title_selector)
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title"))
        })?;

    let mikan_bangumi_id = html
        .select(mikan_bangumi_id_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(|s| mikan_bangumi_homepage_url.join(s).ok())
        .and_then(|rss_link_url| extract_mikan_bangumi_id_from_rss_url(&rss_link_url))
        .map(
            |MikanBangumiRssUrlMeta {
                 mikan_bangumi_id, ..
             }| mikan_bangumi_id,
        )
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_bangumi_id"))
        })?;

    let origin_poster_src = html.select(bangumi_poster_selector).next().and_then(|el| {
        el.value()
            .attr("data-src")
            .and_then(|data_src| extract_image_src_from_str(data_src, mikan_base_url))
            .or_else(|| {
                el.value().attr("style").and_then(|style| {
                    extract_background_image_src_from_style_attr(style, mikan_base_url)
                })
            })
    });

    tracing::trace!(
        bangumi_title,
        mikan_bangumi_id,
        origin_poster_src = origin_poster_src.as_ref().map(|url| url.as_str()),
        "mikan bangumi index meta extracted"
    );

    Ok(MikanBangumiIndexMeta {
        homepage: mikan_bangumi_homepage_url,
        bangumi_title,
        origin_poster_src,
        mikan_bangumi_id,
    })
}

pub fn extract_mikan_fansub_meta_from_bangumi_homepage_html(
    html: &Html,
    mikan_fansub_id: String,
) -> Option<MikanFansubMeta> {
    html.select(
        &Selector::parse(&format!(
            "a.subgroup-name[data-anchor='#{mikan_fansub_id}']"
        ))
        .unwrap(),
    )
    .next()
    .map(extract_inner_text_from_element_ref)
    .map(|fansub_name| MikanFansubMeta {
        mikan_fansub_id,
        fansub: fansub_name,
    })
}

#[instrument(err, skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub fn extract_mikan_bangumi_meta_from_bangumi_homepage_html(
    html: &Html,
    mikan_bangumi_homepage_url: Url,
    mikan_base_url: &Url,
) -> RecorderResult<MikanBangumiMeta> {
    let mikan_fansub_id = MikanBangumiHomepageUrlMeta::from_url(&mikan_bangumi_homepage_url)
        .map(|s| s.mikan_fansub_id)
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_fansub_id"))
        })?;

    let bangumi_index_meta = extract_mikan_bangumi_index_meta_from_bangumi_homepage_html(
        html,
        mikan_bangumi_homepage_url,
        mikan_base_url,
    )?;

    let fansub_meta = extract_mikan_fansub_meta_from_bangumi_homepage_html(html, mikan_fansub_id)
        .ok_or_else(|| {
        RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("fansub_name"))
    })?;

    Ok(MikanBangumiMeta::from_bangumi_index_and_fansub_meta(
        bangumi_index_meta,
        fansub_meta,
    ))
}

#[instrument(err, skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub async fn scrape_mikan_bangumi_meta_from_bangumi_homepage_url(
    mikan_client: &MikanClient,
    mikan_bangumi_homepage_url: Url,
) -> RecorderResult<MikanBangumiMeta> {
    let mikan_base_url = mikan_client.base_url();
    let content = fetch_html(mikan_client, mikan_bangumi_homepage_url.as_str()).await?;
    let html = Html::parse_document(&content);

    extract_mikan_bangumi_meta_from_bangumi_homepage_html(
        &html,
        mikan_bangumi_homepage_url,
        mikan_base_url,
    )
}

#[instrument(err, skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub async fn scrape_mikan_bangumi_index_meta_from_bangumi_homepage_url(
    mikan_client: &MikanClient,
    mikan_bangumi_homepage_url: Url,
) -> RecorderResult<MikanBangumiIndexMeta> {
    let mikan_base_url = mikan_client.base_url();
    let content = fetch_html(mikan_client, mikan_bangumi_homepage_url.as_str()).await?;
    let html = Html::parse_document(&content);

    extract_mikan_bangumi_index_meta_from_bangumi_homepage_html(
        &html,
        mikan_bangumi_homepage_url,
        mikan_base_url,
    )
}

#[instrument(skip_all, fields(origin_poster_src_url = origin_poster_src_url.as_str()))]
pub async fn scrape_mikan_poster_data_from_image_url(
    mikan_client: &MikanClient,
    origin_poster_src_url: Url,
) -> RecorderResult<Bytes> {
    let poster_data = fetch_image(mikan_client, origin_poster_src_url.clone()).await?;
    Ok(poster_data)
}

#[instrument(skip_all, fields(origin_poster_src_url = origin_poster_src_url.as_str()))]
pub async fn scrape_mikan_poster_meta_from_image_url(
    mikan_client: &MikanClient,
    storage_service: &dyn StorageServiceTrait,
    origin_poster_src_url: Url,
    subscriber_id: i32,
) -> RecorderResult<MikanBangumiPosterMeta> {
    if let Some(poster_src) = storage_service
        .exists_object(
            StorageContentCategory::Image,
            subscriber_id,
            Some(MIKAN_POSTER_BUCKET_KEY),
            &origin_poster_src_url.path().replace("/images/Bangumi/", ""),
        )
        .await?
    {
        return Ok(MikanBangumiPosterMeta {
            origin_poster_src: origin_poster_src_url,
            poster_src: Some(poster_src.to_string()),
        });
    }

    let poster_data =
        scrape_mikan_poster_data_from_image_url(mikan_client, origin_poster_src_url.clone())
            .await?;

    let poster_str = storage_service
        .store_object(
            StorageContentCategory::Image,
            subscriber_id,
            Some(MIKAN_POSTER_BUCKET_KEY),
            &origin_poster_src_url.path().replace("/images/Bangumi/", ""),
            poster_data,
        )
        .await?;

    Ok(MikanBangumiPosterMeta {
        origin_poster_src: origin_poster_src_url,
        poster_src: Some(poster_str.to_string()),
    })
}

pub fn extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(
    html: &Html,
    mikan_base_url: &Url,
) -> Vec<MikanBangumiIndexMeta> {
    let bangumi_empty_selector = &Selector::parse(".no-subscribe-bangumi").unwrap();

    if html.select(bangumi_empty_selector).next().is_some() {
        return vec![];
    }

    let bangumi_item_selector = &Selector::parse(".mine.an-box ul.an-ul>li").unwrap();
    let bangumi_poster_span_selector = &Selector::parse("span[data-src][data-bangumiid]").unwrap();
    let bangumi_title_a_selector = &Selector::parse(".an-info-group a.an-text[title]").unwrap();

    let mut items = vec![];
    for bangumi_item in html.select(bangumi_item_selector) {
        let bangumi_poster_span = bangumi_item.select(bangumi_poster_span_selector).next();
        let bangumi_title_a = bangumi_item.select(bangumi_title_a_selector).next();
        if let (Some(bangumi_poster_span), Some(bangumi_title_a)) =
            (bangumi_poster_span, bangumi_title_a)
        {
            let origin_poster_src = bangumi_poster_span
                .attr("data-src")
                .and_then(|data_src| extract_image_src_from_str(data_src, mikan_base_url));
            let bangumi_title = bangumi_title_a
                .attr("title")
                .map(|title| decode_html_entities(&title).trim().to_string());
            let mikan_bangumi_id = bangumi_poster_span
                .attr("data-bangumiid")
                .map(|id| id.to_string());

            if let (Some(bangumi_title), Some(mikan_bangumi_id)) = (bangumi_title, mikan_bangumi_id)
            {
                let homepage = build_mikan_bangumi_homepage_url(
                    mikan_base_url.clone(),
                    &mikan_bangumi_id,
                    None,
                );
                if let Some(origin_poster_src) = origin_poster_src.as_ref() {
                    tracing::trace!(
                        origin_poster_src = origin_poster_src.as_str(),
                        bangumi_title,
                        mikan_bangumi_id,
                        "bangumi index meta extracted"
                    );
                } else {
                    tracing::warn!(
                        bangumi_title,
                        mikan_bangumi_id,
                        "bangumi index meta extracted, but failed to extract poster_src"
                    );
                }
                items.push(MikanBangumiIndexMeta {
                    homepage,
                    origin_poster_src,
                    bangumi_title,
                    mikan_bangumi_id,
                });
            }
        }
    }
    items
}

#[instrument(skip_all, fields(mikan_bangumi_index = mikan_bangumi_index.mikan_bangumi_id.as_str()))]
pub fn extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
    html: &Html,
    mikan_bangumi_index: MikanBangumiIndexMeta,
    mikan_base_url: Url,
) -> Option<MikanBangumiMeta> {
    let fansub_container_selector =
        &Selector::parse(".js-expand_bangumi-subgroup.js-subscribed").unwrap();
    let fansub_title_selector = &Selector::parse(".tag-res-name[title]").unwrap();
    let fansub_id_selector =
        &Selector::parse(".active[data-subtitlegroupid][data-bangumiid]").unwrap();

    if let Some((fansub_name, mikan_fansub_id)) = {
        html.select(fansub_container_selector)
            .next()
            .and_then(|fansub_info| {
                if let (Some(fansub_name), Some(mikan_fansub_id)) = (
                    fansub_info
                        .select(fansub_title_selector)
                        .next()
                        .and_then(|ele| ele.attr("title"))
                        .map(String::from),
                    fansub_info
                        .select(fansub_id_selector)
                        .next()
                        .and_then(|ele| ele.attr("data-subtitlegroupid"))
                        .map(String::from),
                ) {
                    Some((fansub_name, mikan_fansub_id))
                } else {
                    None
                }
            })
    } {
        tracing::trace!(
            mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id,
            bangumi_title = mikan_bangumi_index.bangumi_title,
            fansub_name,
            mikan_fansub_id,
            "subscribed fansub extracted"
        );

        let mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id;
        let bangumi_title = mikan_bangumi_index.bangumi_title;
        let origin_poster_src = mikan_bangumi_index.origin_poster_src;

        Some(MikanBangumiMeta {
            homepage: build_mikan_bangumi_homepage_url(
                mikan_base_url,
                &mikan_bangumi_id,
                Some(&mikan_fansub_id),
            ),
            bangumi_title: bangumi_title.to_string(),
            mikan_bangumi_id: mikan_bangumi_id.to_string(),
            mikan_fansub_id: mikan_fansub_id.to_string(),
            fansub: fansub_name.to_string(),
            origin_poster_src: origin_poster_src.clone(),
        })
    } else {
        tracing::trace!(
            mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id,
            bangumi_title = mikan_bangumi_index.bangumi_title,
            "subscribed fansub failed to extract"
        );
        None
    }
}

pub fn scrape_mikan_bangumi_meta_stream_from_season_flow_url(
    ctx: Arc<dyn AppContextTrait>,
    mikan_season_flow_url: Url,
    credential_id: i32,
) -> impl Stream<Item = RecorderResult<MikanBangumiMeta>> {
    try_stream! {
        let mikan_client = ctx.mikan()
        .fork_with_credential(ctx.clone(), credential_id)
        .await?;

        let mikan_base_url = mikan_client.base_url();
        let content = fetch_html(&mikan_client, mikan_season_flow_url.clone()).await?;
        let mut bangumi_indices_meta = {
            let html = Html::parse_document(&content);
            extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, mikan_base_url)
        };

        if bangumi_indices_meta.is_empty() && !mikan_client.has_login().await? {
            mikan_client.login().await?;
            let content = fetch_html(&mikan_client, mikan_season_flow_url).await?;
            let html = Html::parse_document(&content);
            bangumi_indices_meta =
                extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, mikan_base_url);
        }


        mikan_client
            .sync_credential_cookies(ctx.clone(), credential_id)
            .await?;

        for bangumi_index in bangumi_indices_meta {
            let bangumi_title = bangumi_index.bangumi_title.clone();
            let bangumi_expand_subscribed_fragment_url = build_mikan_bangumi_expand_subscribed_url(
                mikan_base_url.clone(),
                &bangumi_index.mikan_bangumi_id,
            );
            let bangumi_expand_subscribed_fragment =
                fetch_html(&mikan_client, bangumi_expand_subscribed_fragment_url).await?;

            let bangumi_meta = {
                let html = Html::parse_document(&bangumi_expand_subscribed_fragment);

                extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
                    &html,
                    bangumi_index,
                    mikan_base_url.clone(),
                )
                .with_whatever_context::<_, String, RecorderError>(|| {
                    format!("failed to extract mikan bangumi fansub of title = {bangumi_title}")
                })
            }?;

            yield bangumi_meta;
        }

        mikan_client
        .sync_credential_cookies(ctx, credential_id)
        .await?;
    }
}

#[instrument(err, skip_all, fields(mikan_season_flow_url = mikan_season_flow_url.as_str(), credential_id = credential_id))]
pub async fn scrape_mikan_bangumi_meta_list_from_season_flow_url(
    _mikan_client: &MikanClient,
    ctx: Arc<dyn AppContextTrait>,
    mikan_season_flow_url: Url,
    credential_id: i32,
) -> RecorderResult<Vec<MikanBangumiMeta>> {
    let stream = scrape_mikan_bangumi_meta_stream_from_season_flow_url(
        ctx,
        mikan_season_flow_url,
        credential_id,
    );

    pin_mut!(stream);

    let bangumi_metas = stream.try_collect().await?;

    Ok(bangumi_metas)
}

#[cfg(test)]
mod test {
    #![allow(unused_variables)]
    use std::fs;

    use rstest::{fixture, rstest};
    use tracing::Level;
    use url::Url;
    use zune_image::{codecs::ImageFormat, image::Image};

    use super::*;
    use crate::{
        extract::mikan::{MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH, MIKAN_SEASON_FLOW_PAGE_PATH},
        test_utils::{
            app::UnitTestAppContext,
            crypto::build_testing_crypto_service,
            database::build_testing_database_service,
            mikan::{
                MikanMockServer, build_testing_mikan_client, build_testing_mikan_credential_form,
            },
            storage::build_testing_storage_service,
            tracing::try_init_testing_tracing,
        },
    };

    #[fixture]
    fn before_each() {
        try_init_testing_tracing(Level::DEBUG);
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrape_mikan_poster_data_from_image_url(before_each: ()) -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;
        let mikan_base_url = Url::parse(&mikan_server.url())?;
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        let bangumi_poster_url = mikan_base_url.join("/images/Bangumi/202309/5ce9fed1.jpg")?;

        let bangumi_poster_mock = mikan_server
            .mock("GET", bangumi_poster_url.path())
            .with_body_from_file("tests/resources/mikan/Bangumi-202309-5ce9fed1.jpg")
            .create_async()
            .await;

        let bgm_poster_data =
            scrape_mikan_poster_data_from_image_url(&mikan_client, bangumi_poster_url).await?;

        bangumi_poster_mock.expect(1);
        let image = Image::read(bgm_poster_data.to_vec(), Default::default());
        assert!(
            image.is_ok_and(|img| img
                .metadata()
                .get_image_format()
                .is_some_and(|fmt| matches!(fmt, ImageFormat::JPEG))),
            "should start with valid jpeg data magic number"
        );

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrape_mikan_poster_meta_from_image_url(before_each: ()) -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;
        let mikan_base_url = Url::parse(&mikan_server.url())?;
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;
        let storage_service = build_testing_storage_service().await?;
        let storage_operator = storage_service.get_operator()?;

        let bangumi_poster_url = mikan_base_url.join("/images/Bangumi/202309/5ce9fed1.jpg")?;

        let bangumi_poster_mock = mikan_server
            .mock("GET", bangumi_poster_url.path())
            .with_body_from_file("tests/resources/mikan/Bangumi-202309-5ce9fed1.jpg")
            .create_async()
            .await;

        let bgm_poster = scrape_mikan_poster_meta_from_image_url(
            &mikan_client,
            &storage_service,
            bangumi_poster_url,
            1,
        )
        .await?;

        bangumi_poster_mock.expect(1);

        let storage_fullname = storage_service.get_fullname(
            StorageContentCategory::Image,
            1,
            Some(MIKAN_POSTER_BUCKET_KEY),
            "202309/5ce9fed1.jpg",
        );
        let storage_fullename_str = storage_fullname.as_str();

        assert!(storage_operator.exists(storage_fullename_str).await?);

        let expected_data = fs::read("tests/resources/mikan/Bangumi-202309-5ce9fed1.jpg")?;
        let found_data = storage_operator.read(storage_fullename_str).await?.to_vec();
        assert_eq!(expected_data, found_data);

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(
        before_each: (),
    ) -> RecorderResult<()> {
        let fragment_str =
            fs::read_to_string("tests/resources/mikan/BangumiCoverFlow-2025-spring.html")?;

        let mikan_base_url = Url::parse("https://mikanani.me/")?;

        let bangumi_index_meta_list =
            extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(
                &Html::parse_document(&fragment_str),
                &mikan_base_url,
            );

        assert_eq!(bangumi_index_meta_list.len(), 49);
        let first = &bangumi_index_meta_list[0];
        assert_eq!(first.bangumi_title, "吉伊卡哇");
        assert_eq!(first.mikan_bangumi_id, "3288");
        assert_eq!(
            first.homepage.to_string(),
            String::from("https://mikanani.me/Home/Bangumi/3288")
        );
        assert_eq!(
            first
                .origin_poster_src
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            String::from("https://mikanani.me/images/Bangumi/202204/d8ef46c0.jpg")
        );

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_index_meta_list_from_season_flow_fragment_noauth(
        before_each: (),
    ) -> RecorderResult<()> {
        let fragment_str =
            fs::read_to_string("tests/resources/mikan/BangumiCoverFlow-2025-spring-noauth.html")?;

        let bangumi_index_meta_list =
            extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(
                &Html::parse_document(&fragment_str),
                &Url::parse("https://mikanani.me/")?,
            );

        assert!(bangumi_index_meta_list.is_empty());

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
        before_each: (),
    ) -> RecorderResult<()> {
        let origin_poster_src =
            Url::parse("https://mikanani.me/images/Bangumi/202504/076c1094.jpg")?;
        let bangumi_index_meta = MikanBangumiIndexMeta {
            homepage: Url::parse("https://mikanani.me/Home/Bangumi/3599")?,
            origin_poster_src: Some(origin_poster_src.clone()),
            bangumi_title: "夏日口袋".to_string(),
            mikan_bangumi_id: "3599".to_string(),
        };

        let fragment_str = fs::read_to_string("tests/resources/mikan/ExpandBangumi-3599.html")?;

        let bangumi = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
            &Html::parse_document(&fragment_str),
            bangumi_index_meta.clone(),
            Url::parse("https://mikanani.me/")?,
        )
        .unwrap_or_else(|| {
            panic!("bangumi should not be None");
        });

        assert_eq!(
            bangumi.homepage,
            Url::parse("https://mikanani.me/Home/Bangumi/3599#370")?
        );
        assert_eq!(bangumi.bangumi_title, bangumi_index_meta.bangumi_title);
        assert_eq!(
            bangumi.mikan_bangumi_id,
            bangumi_index_meta.mikan_bangumi_id
        );
        assert_eq!(
            bangumi.origin_poster_src,
            bangumi_index_meta.origin_poster_src
        );
        assert_eq!(bangumi.mikan_fansub_id, String::from("370"));
        assert_eq!(bangumi.fansub, String::from("LoliHouse"));

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_meta_from_expand_subscribed_fragment_noauth(
        before_each: (),
    ) -> RecorderResult<()> {
        let origin_poster_src =
            Url::parse("https://mikanani.me/images/Bangumi/202504/076c1094.jpg")?;
        let bangumi_index_meta = MikanBangumiIndexMeta {
            homepage: Url::parse("https://mikanani.me/Home/Bangumi/3599")?,
            origin_poster_src: Some(origin_poster_src.clone()),
            bangumi_title: "夏日口袋".to_string(),
            mikan_bangumi_id: "3599".to_string(),
        };

        let fragment_str =
            fs::read_to_string("tests/resources/mikan/ExpandBangumi-3599-noauth.html")?;

        let bangumi = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
            &Html::parse_document(&fragment_str),
            bangumi_index_meta.clone(),
            Url::parse("https://mikanani.me/")?,
        );

        assert!(bangumi.is_none());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrape_mikan_bangumi_meta_list_from_season_flow_url(
        before_each: (),
    ) -> RecorderResult<()> {
        let mut mikan_server = MikanMockServer::new().await?;
        let mikan_base_url = mikan_server.base_url().clone();

        let app_ctx = {
            let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;
            let db_service = build_testing_database_service().await?;
            let crypto_service = build_testing_crypto_service().await?;
            let app_ctx = UnitTestAppContext::builder()
                .mikan(mikan_client)
                .db(db_service)
                .crypto(crypto_service)
                .build();

            Arc::new(app_ctx)
        };

        let mikan_client = app_ctx.mikan();

        let login_mock = mikan_server.mock_get_login_page();

        let season_flow_noauth_mock = mikan_server
            .server
            .mock("GET", MIKAN_SEASON_FLOW_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(|req| !MikanMockServer::get_has_auth_matcher()(req))
            .with_status(200)
            .with_body_from_file("tests/resources/mikan/BangumiCoverFlow-2025-spring-noauth.html")
            .create();

        let season_flow_mock = mikan_server
            .server
            .mock("GET", MIKAN_SEASON_FLOW_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(|req| MikanMockServer::get_has_auth_matcher()(req))
            .with_status(200)
            .with_body_from_file("tests/resources/mikan/BangumiCoverFlow-2025-spring.html")
            .create();

        let bangumi_subscribed_noauth_mock = mikan_server
            .server
            .mock("GET", MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(|req| !MikanMockServer::get_has_auth_matcher()(req))
            .with_status(200)
            .with_body_from_file("tests/resources/mikan/ExpandBangumi-3599-noauth.html")
            .create();

        let bangumi_subscribed_mock = mikan_server
            .server
            .mock("GET", MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH)
            .match_query(mockito::Matcher::Any)
            .match_request(|req| MikanMockServer::get_has_auth_matcher()(req))
            .with_status(200)
            .with_body_from_file("tests/resources/mikan/ExpandBangumi-3599.html")
            .create();

        let credential = mikan_client
            .submit_credential_form(app_ctx.clone(), 1, build_testing_mikan_credential_form())
            .await?;

        let mikan_season_flow_url =
            build_mikan_season_flow_url(mikan_base_url.clone(), 2025, MikanSeasonStr::Spring);

        let bangumi_meta_list = scrape_mikan_bangumi_meta_list_from_season_flow_url(
            mikan_client,
            app_ctx.clone(),
            mikan_season_flow_url,
            credential.id,
        )
        .await?;

        assert!(!bangumi_meta_list.is_empty());

        let bangumi = bangumi_meta_list.first().unwrap();

        assert!(
            bangumi
                .homepage
                .to_string()
                .ends_with("/Home/Bangumi/3288#370"),
        );
        assert_eq!(bangumi.bangumi_title, "吉伊卡哇");
        assert_eq!(bangumi.mikan_bangumi_id, "3288");
        assert!(
            bangumi
                .origin_poster_src
                .as_ref()
                .map_or(String::new(), |u| u.to_string())
                .ends_with("/images/Bangumi/202204/d8ef46c0.jpg")
        );
        assert_eq!(bangumi.mikan_fansub_id, String::from("370"));
        assert_eq!(bangumi.fansub, String::from("LoliHouse"));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrape_mikan_episode_meta_from_episode_homepage_url(
        before_each: (),
    ) -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;
        let mikan_base_url = Url::parse(&mikan_server.url())?;
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        let episode_homepage_url = mikan_base_url
            .clone()
            .join("/Home/Episode/475184dce83ea2b82902592a5ac3343f6d54b36a")?;

        let episode_homepage_mock = mikan_server
            .mock("GET", episode_homepage_url.path())
            .with_body_from_file(
                "tests/resources/mikan/Episode-475184dce83ea2b82902592a5ac3343f6d54b36a.htm",
            )
            .create_async()
            .await;

        let episode_meta = scrape_mikan_episode_meta_from_episode_homepage_url(
            &mikan_client,
            episode_homepage_url.clone(),
        )
        .await?;

        assert_eq!(episode_meta.homepage, episode_homepage_url);
        assert_eq!(episode_meta.bangumi_title, "葬送的芙莉莲");
        assert_eq!(
            episode_meta
                .origin_poster_src
                .as_ref()
                .map(|s| s.path().to_string()),
            Some(String::from("/images/Bangumi/202309/5ce9fed1.jpg"))
        );
        assert_eq!(episode_meta.fansub, "LoliHouse");
        assert_eq!(episode_meta.mikan_fansub_id, "370");
        assert_eq!(episode_meta.mikan_bangumi_id, "3141");

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrape_mikan_bangumi_meta_from_bangumi_homepage_url(
        before_each: (),
    ) -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;
        let mikan_base_url = Url::parse(&mikan_server.url())?;
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        let bangumi_homepage_url = mikan_base_url.join("/Home/Bangumi/3416#370")?;

        let bangumi_homepage_mock = mikan_server
            .mock("GET", bangumi_homepage_url.path())
            .with_body_from_file("tests/resources/mikan/Bangumi-3416-370.htm")
            .create_async()
            .await;

        let bangumi_meta = scrape_mikan_bangumi_meta_from_bangumi_homepage_url(
            &mikan_client,
            bangumi_homepage_url.clone(),
        )
        .await?;

        assert_eq!(bangumi_meta.homepage, bangumi_homepage_url);
        assert_eq!(bangumi_meta.bangumi_title, "叹气的亡灵想隐退");
        assert_eq!(
            bangumi_meta
                .origin_poster_src
                .as_ref()
                .map(|s| s.path().to_string()),
            Some(String::from("/images/Bangumi/202410/480ef127.jpg"))
        );
        assert_eq!(bangumi_meta.fansub, String::from("LoliHouse"));
        assert_eq!(bangumi_meta.mikan_fansub_id, String::from("370"));
        assert_eq!(bangumi_meta.mikan_bangumi_id, "3416");

        Ok(())
    }
}
