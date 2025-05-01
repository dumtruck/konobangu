use std::{borrow::Cow, fmt};

use bytes::Bytes;
use fetch::{html::fetch_html, image::fetch_image};
use html_escape::decode_html_entities;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

use super::{
    MIKAN_BUCKET_KEY, MikanBangumiRssUrlMeta, MikanClient, extract_mikan_bangumi_id_from_rss_url,
};
use crate::{
    app::AppContextTrait,
    errors::app_error::{RecorderError, RecorderResult},
    extract::{
        html::{extract_background_image_src_from_style_attr, extract_inner_text_from_element_ref},
        media::extract_image_src_from_str,
    },
    storage::StorageContentCategory,
};

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum MikanSeasonStr {
    #[serde(rename = "春")]
    Spring,
    #[serde(rename = "夏")]
    Summer,
    #[serde(rename = "秋")]
    Autumn,
    #[serde(rename = "冬")]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MikanBangumiIndexMeta {
    pub homepage: Url,
    pub origin_poster_src: Option<Url>,
    pub bangumi_title: String,
    pub mikan_bangumi_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MikanBangumiMeta {
    pub homepage: Url,
    pub origin_poster_src: Option<Url>,
    pub bangumi_title: String,
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: Option<String>,
    pub fansub: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiPosterMeta {
    pub origin_poster_src: Url,
    pub poster_data: Option<Bytes>,
    pub poster_src: Option<String>,
}

impl From<MikanBangumiIndexMeta> for MikanBangumiMeta {
    fn from(index_meta: MikanBangumiIndexMeta) -> Self {
        MikanBangumiMeta {
            homepage: index_meta.homepage,
            origin_poster_src: index_meta.origin_poster_src,
            bangumi_title: index_meta.bangumi_title,
            mikan_bangumi_id: index_meta.mikan_bangumi_id,
            mikan_fansub_id: None,
            fansub: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpisodeHomepage {
    pub mikan_episode_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiHomepage {
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: Option<String>,
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
    url.set_path("/Home/BangumiCoverFlow");
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

pub fn build_mikan_bangumi_expand_subscribed_fragment_url(
    mikan_base_url: Url,
    mikan_bangumi_id: &str,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path("/ExpandBangumi");
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id)
        .append_pair("showSubscribed", "true");
    url
}

pub fn extract_mikan_bangumi_id_from_homepage_url(url: &Url) -> Option<MikanBangumiHomepage> {
    if url.path().starts_with("/Home/Bangumi/") {
        let mikan_bangumi_id = url.path().replace("/Home/Bangumi/", "");

        Some(MikanBangumiHomepage {
            mikan_bangumi_id,
            mikan_fansub_id: url.fragment().map(String::from),
        })
    } else {
        None
    }
}

pub fn extract_mikan_episode_id_from_homepage_url(url: &Url) -> Option<MikanEpisodeHomepage> {
    if url.path().starts_with("/Home/Episode/") {
        let mikan_episode_id = url.path().replace("/Home/Episode/", "");
        Some(MikanEpisodeHomepage { mikan_episode_id })
    } else {
        None
    }
}

pub async fn extract_mikan_poster_meta_from_src(
    http_client: &MikanClient,
    origin_poster_src_url: Url,
) -> Result<MikanBangumiPosterMeta, RecorderError> {
    let poster_data = fetch_image(http_client, origin_poster_src_url.clone()).await?;
    Ok(MikanBangumiPosterMeta {
        origin_poster_src: origin_poster_src_url,
        poster_data: Some(poster_data),
        poster_src: None,
    })
}

pub async fn extract_mikan_bangumi_poster_meta_from_src_with_cache(
    ctx: &dyn AppContextTrait,
    origin_poster_src_url: Url,
    subscriber_id: i32,
) -> RecorderResult<MikanBangumiPosterMeta> {
    let dal_client = ctx.storage();
    let mikan_client = ctx.mikan();
    if let Some(poster_src) = dal_client
        .exists_object(
            StorageContentCategory::Image,
            subscriber_id,
            Some(MIKAN_BUCKET_KEY),
            &origin_poster_src_url.path().replace("/images/Bangumi/", ""),
        )
        .await?
    {
        return Ok(MikanBangumiPosterMeta {
            origin_poster_src: origin_poster_src_url,
            poster_data: None,
            poster_src: Some(poster_src.to_string()),
        });
    }

    let poster_data = fetch_image(mikan_client, origin_poster_src_url.clone()).await?;

    let poster_str = dal_client
        .store_object(
            StorageContentCategory::Image,
            subscriber_id,
            Some(MIKAN_BUCKET_KEY),
            &origin_poster_src_url.path().replace("/images/Bangumi/", ""),
            poster_data.clone(),
        )
        .await?;

    Ok(MikanBangumiPosterMeta {
        origin_poster_src: origin_poster_src_url,
        poster_data: Some(poster_data),
        poster_src: Some(poster_str.to_string()),
    })
}

#[instrument(skip_all, fields(mikan_episode_homepage_url = mikan_episode_homepage_url.as_str()))]
pub async fn extract_mikan_episode_meta_from_episode_homepage(
    http_client: &MikanClient,
    mikan_episode_homepage_url: Url,
) -> Result<MikanEpisodeMeta, RecorderError> {
    let mikan_base_url = Url::parse(&mikan_episode_homepage_url.origin().unicode_serialization())?;
    let content = fetch_html(http_client, mikan_episode_homepage_url.as_str()).await?;

    let html = Html::parse_document(&content);

    let bangumi_title_selector =
        &Selector::parse(".bangumi-title > a[href^='/Home/Bangumi/']").unwrap();
    let mikan_bangumi_id_selector =
        &Selector::parse(".bangumi-title > a.mikan-rss[data-original-title='RSS']").unwrap();
    let bangumi_poster_selector = &Selector::parse(".bangumi-poster").unwrap();

    let bangumi_title = html
        .select(bangumi_title_selector)
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title")))
        .inspect_err(|error| {
            tracing::warn!(error = %error);
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
        })
        .inspect_err(|error| tracing::error!(error = %error))?;

    let mikan_fansub_id = mikan_fansub_id
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_fansub_id"))
        })
        .inspect_err(|error| tracing::error!(error = %error))?;

    let episode_title = html
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("episode_title")))
        .inspect_err(|error| {
            tracing::warn!(error = %error);
        })?;

    let MikanEpisodeHomepage {
        mikan_episode_id, ..
    } = extract_mikan_episode_id_from_homepage_url(&mikan_episode_homepage_url)
        .ok_or_else(|| {
            RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_episode_id"))
        })
        .inspect_err(|error| {
            tracing::warn!(error = %error);
        })?;

    let fansub_name = html
        .select(
            &Selector::parse(".bangumi-info a.magnet-link-wrap[href^='/Home/PublishGroup/']")
                .unwrap(),
        )
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("fansub_name")))
        .inspect_err(|error| {
            tracing::warn!(error = %error);
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

#[instrument(skip_all, fields(mikan_bangumi_homepage_url = mikan_bangumi_homepage_url.as_str()))]
pub async fn extract_mikan_bangumi_meta_from_bangumi_homepage(
    http_client: &MikanClient,
    mikan_bangumi_homepage_url: Url,
) -> Result<MikanBangumiMeta, RecorderError> {
    let mikan_base_url = Url::parse(&mikan_bangumi_homepage_url.origin().unicode_serialization())?;
    let content = fetch_html(http_client, mikan_bangumi_homepage_url.as_str()).await?;
    let html = Html::parse_document(&content);

    let bangumi_title_selector = &Selector::parse(".bangumi-title").unwrap();
    let mikan_bangumi_id_selector =
        &Selector::parse(".bangumi-title > .mikan-rss[data-original-title='RSS']").unwrap();
    let bangumi_poster_selector = &Selector::parse(".bangumi-poster").unwrap();

    let bangumi_title = html
        .select(bangumi_title_selector)
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| RecorderError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title")))
        .inspect_err(|error| tracing::warn!(error = %error))?;

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
        })
        .inspect_err(|error| tracing::error!(error = %error))?;

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

    let (mikan_fansub_id, fansub) = mikan_bangumi_homepage_url
        .fragment()
        .and_then(|id| {
            html.select(
                &Selector::parse(&format!("a.subgroup-name[data-anchor='#{}']", id)).unwrap(),
            )
            .next()
            .map(extract_inner_text_from_element_ref)
            .map(|fansub_name| (id.to_string(), fansub_name))
        })
        .unzip();

    tracing::trace!(
        bangumi_title,
        mikan_bangumi_id,
        origin_poster_src = origin_poster_src.as_ref().map(|url| url.as_str()),
        fansub,
        mikan_fansub_id,
        "mikan bangumi meta extracted"
    );

    Ok(MikanBangumiMeta {
        homepage: mikan_bangumi_homepage_url,
        bangumi_title,
        origin_poster_src,
        mikan_bangumi_id,
        fansub,
        mikan_fansub_id,
    })
}

#[instrument]
pub fn extract_mikan_bangumi_indices_meta_from_season_flow_fragment(
    season_flow_fragment: &str,
    mikan_base_url: Url,
) -> Vec<MikanBangumiIndexMeta> {
    let html = Html::parse_fragment(season_flow_fragment);

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
                .and_then(|data_src| extract_image_src_from_str(data_src, &mikan_base_url));
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
                })
            }
        }
    }
    items
}

#[instrument(skip_all, fields(mikan_bangumi_index = mikan_bangumi_index.mikan_bangumi_id.as_str()))]
pub fn extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
    mikan_bangumi_index: MikanBangumiIndexMeta,
    expand_subscribed_fragment: &str,
    mikan_base_url: Url,
) -> Option<MikanBangumiMeta> {
    let html = Html::parse_fragment(expand_subscribed_fragment);
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
        tracing::trace!(fansub_name, mikan_fansub_id, "subscribed fansub extracted");
        let mikan_bangumi_id = mikan_bangumi_index.mikan_bangumi_id;
        let bangumi_title = mikan_bangumi_index.bangumi_title;
        let origin_poster_src = mikan_bangumi_index.origin_poster_src;

        Some(MikanBangumiMeta {
            homepage: build_mikan_bangumi_homepage_url(
                mikan_base_url.clone(),
                &mikan_bangumi_id,
                Some(&mikan_fansub_id),
            ),
            bangumi_title: bangumi_title.to_string(),
            mikan_bangumi_id: mikan_bangumi_id.to_string(),
            mikan_fansub_id: Some(mikan_fansub_id),
            fansub: Some(fansub_name),
            origin_poster_src: origin_poster_src.clone(),
        })
    } else {
        tracing::trace!("subscribed fansub not found");
        None
    }
}

#[cfg(test)]
mod test {
    #![allow(unused_variables)]
    use std::{fs, sync::Arc};

    use futures::{TryStreamExt, pin_mut};
    use http::header;
    use rstest::{fixture, rstest};
    use tracing::Level;
    use url::Url;
    use zune_image::{codecs::ImageFormat, image::Image};

    use super::*;
    use crate::{
        extract::mikan::MikanCredentialForm,
        test_utils::{
            app::UnitTestAppContext, mikan::build_testing_mikan_client,
            tracing::try_init_testing_tracing,
        },
    };

    #[fixture]
    fn before_each() {
        try_init_testing_tracing(Level::INFO);
    }

    #[rstest]
    #[tokio::test]
    async fn test_extract_mikan_poster_from_src(before_each: ()) -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;
        let mikan_base_url = Url::parse(&mikan_server.url())?;
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        let bangumi_poster_url = mikan_base_url.join("/images/Bangumi/202309/5ce9fed1.jpg")?;

        let bangumi_poster_mock = mikan_server
            .mock("GET", bangumi_poster_url.path())
            .with_body_from_file("tests/resources/mikan/Bangumi-202309-5ce9fed1.jpg")
            .create_async()
            .await;

        let bgm_poster =
            extract_mikan_poster_meta_from_src(&mikan_client, bangumi_poster_url).await?;
        bangumi_poster_mock.expect(1);
        let u8_data = bgm_poster.poster_data.expect("should have poster data");
        let image = Image::read(u8_data.to_vec(), Default::default());
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
    async fn test_extract_mikan_episode(before_each: ()) -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;
        let mikan_base_url = Url::parse(&mikan_server.url())?;
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        let episode_homepage_url =
            mikan_base_url.join("/Home/Episode/475184dce83ea2b82902592a5ac3343f6d54b36a")?;

        let episode_homepage_mock = mikan_server
            .mock("GET", episode_homepage_url.path())
            .with_body_from_file(
                "tests/resources/mikan/Episode-475184dce83ea2b82902592a5ac3343f6d54b36a.htm",
            )
            .create_async()
            .await;

        let ep_meta = extract_mikan_episode_meta_from_episode_homepage(
            &mikan_client,
            episode_homepage_url.clone(),
        )
        .await?;

        assert_eq!(ep_meta.homepage, episode_homepage_url);
        assert_eq!(ep_meta.bangumi_title, "葬送的芙莉莲");
        assert_eq!(
            ep_meta
                .origin_poster_src
                .as_ref()
                .map(|s| s.path().to_string()),
            Some(String::from("/images/Bangumi/202309/5ce9fed1.jpg"))
        );
        assert_eq!(ep_meta.fansub, "LoliHouse");
        assert_eq!(ep_meta.mikan_fansub_id, "370");
        assert_eq!(ep_meta.mikan_bangumi_id, "3141");

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_extract_mikan_bangumi_meta_from_bangumi_homepage(
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

        let bgm_meta = extract_mikan_bangumi_meta_from_bangumi_homepage(
            &mikan_client,
            bangumi_homepage_url.clone(),
        )
        .await?;

        assert_eq!(bgm_meta.homepage, bangumi_homepage_url);
        assert_eq!(bgm_meta.bangumi_title, "叹气的亡灵想隐退");
        assert_eq!(
            bgm_meta
                .origin_poster_src
                .as_ref()
                .map(|s| s.path().to_string()),
            Some(String::from("/images/Bangumi/202410/480ef127.jpg"))
        );
        assert_eq!(bgm_meta.fansub, Some(String::from("LoliHouse")));
        assert_eq!(bgm_meta.mikan_fansub_id, Some(String::from("370")));
        assert_eq!(bgm_meta.mikan_bangumi_id, "3416");

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_indices_meta_from_season_flow_fragment(
        before_each: (),
    ) -> RecorderResult<()> {
        let fragment =
            fs::read_to_string("tests/resources/mikan/BangumiCoverFlow-2025-spring.html")?;

        let indices = extract_mikan_bangumi_indices_meta_from_season_flow_fragment(
            &fragment,
            Url::parse("https://mikanani.me/")?,
        );

        tracing::info!("indices: {:#?}", &indices[0]);

        assert_eq!(indices.len(), 49);
        let first = &indices[0];
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
    fn test_extract_mikan_bangumi_indices_meta_from_season_flow_fragment_noauth(
        before_each: (),
    ) -> RecorderResult<()> {
        let fragment =
            fs::read_to_string("tests/resources/mikan/BangumiCoverFlow-2025-spring-noauth.html")?;

        let indices = extract_mikan_bangumi_indices_meta_from_season_flow_fragment(
            &fragment,
            Url::parse("https://mikanani.me/")?,
        );

        assert!(indices.is_empty());

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
        before_each: (),
    ) -> RecorderResult<()> {
        let origin_poster_src =
            Url::parse("https://mikanani.me/images/Bangumi/202504/076c1094.jpg")?;
        let bangumi_index = MikanBangumiIndexMeta {
            homepage: Url::parse("https://mikanani.me/Home/Bangumi/3599")?,
            origin_poster_src: Some(origin_poster_src.clone()),
            bangumi_title: "夏日口袋".to_string(),
            mikan_bangumi_id: "3599".to_string(),
        };

        let fragment = fs::read_to_string("tests/resources/mikan/ExpandBangumi-3599.html")?;

        let bangumi = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
            bangumi_index.clone(),
            &fragment,
            Url::parse("https://mikanani.me/")?,
        )
        .unwrap_or_else(|| {
            panic!("bangumi should not be None");
        });

        assert_eq!(
            bangumi.homepage,
            Url::parse("https://mikanani.me/Home/Bangumi/3599#370")?
        );
        assert_eq!(bangumi.bangumi_title, bangumi_index.bangumi_title);
        assert_eq!(bangumi.mikan_bangumi_id, bangumi_index.mikan_bangumi_id);
        assert_eq!(bangumi.origin_poster_src, bangumi_index.origin_poster_src);
        assert_eq!(bangumi.mikan_fansub_id, Some(String::from("370")));
        assert_eq!(bangumi.fansub, Some(String::from("LoliHouse")));

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_extract_mikan_bangumi_meta_from_expand_subscribed_fragment_noauth(
        before_each: (),
    ) -> RecorderResult<()> {
        let origin_poster_src =
            Url::parse("https://mikanani.me/images/Bangumi/202504/076c1094.jpg")?;
        let bangumi_index = MikanBangumiIndexMeta {
            homepage: Url::parse("https://mikanani.me/Home/Bangumi/3599")?,
            origin_poster_src: Some(origin_poster_src.clone()),
            bangumi_title: "夏日口袋".to_string(),
            mikan_bangumi_id: "3599".to_string(),
        };

        let fragment = fs::read_to_string("tests/resources/mikan/ExpandBangumi-3599-noauth.html")?;

        let bangumi = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
            bangumi_index.clone(),
            &fragment,
            Url::parse("https://mikanani.me/")?,
        );

        assert!(bangumi.is_none());

        Ok(())
    }

    // #[rstest]
    // #[tokio::test]
    // async fn test_extract_mikan_bangumis_meta_from_my_bangumi_page(
    //     before_each: (),
    // ) -> RecorderResult<()> {
    //     let mut mikan_server = mockito::Server::new_async().await;

    //     let mikan_base_url = Url::parse(&mikan_server.url())?;

    //     let my_bangumi_page_url = mikan_base_url.join("/Home/MyBangumi")?;

    //     let context = Arc::new(
    //         UnitTestAppContext::builder()
    //
    // .mikan(build_testing_mikan_client(mikan_base_url.clone()).await?)
    //             .build(),
    //     );

    //     {
    //         let my_bangumi_without_cookie_mock = mikan_server
    //             .mock("GET", my_bangumi_page_url.path())
    //             .match_header(header::COOKIE, mockito::Matcher::Missing)
    //
    // .with_body_from_file("tests/resources/mikan/MyBangumi-noauth.htm")
    //             .create_async()
    //             .await;

    //         let bangumi_metas =
    // extract_mikan_bangumis_meta_from_my_bangumi_page(
    // context.clone(),             my_bangumi_page_url.clone(),
    //             None,
    //             &[],
    //         );

    //         pin_mut!(bangumi_metas);

    //         let bangumi_metas = bangumi_metas.try_collect::<Vec<_>>().await?;

    //         assert!(bangumi_metas.is_empty());

    //         assert!(my_bangumi_without_cookie_mock.matched_async().await);
    //     }
    //     {
    //         let my_bangumi_with_cookie_mock = mikan_server
    //             .mock("GET", my_bangumi_page_url.path())
    //             .match_header(
    //                 header::COOKIE,
    //                 mockito::Matcher::AllOf(vec![
    //
    // mockito::Matcher::Regex(String::from(".*\\.AspNetCore\\.Antiforgery.*")),
    //                     mockito::Matcher::Regex(String::from(
    //                         ".*\\.AspNetCore\\.Identity\\.Application.*",
    //                     )),
    //                 ]),
    //             )
    //             .with_body_from_file("tests/resources/mikan/MyBangumi.htm")
    //             .create_async()
    //             .await;

    //         let expand_bangumi_mock = mikan_server
    //             .mock("GET", "/ExpandBangumi")
    //             .match_query(mockito::Matcher::Any)
    //
    // .with_body_from_file("tests/resources/mikan/ExpandBangumi.htm")
    //             .create_async()
    //             .await;

    //         let auth_secrecy = Some(MikanCredentialForm {
    //             username: String::from("test_username"),
    //             password: String::from("test_password"),
    //             user_agent: String::from(
    //                 "Mozilla/5.0 (Windows NT 10.0; Win64; x64)
    // AppleWebKit/537.36 (KHTML, like \                  Gecko)
    // Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0",             ),
    //         });

    //         let bangumi_metas =
    // extract_mikan_bangumis_meta_from_my_bangumi_page(
    // context.clone(),             my_bangumi_page_url,
    //             auth_secrecy,
    //             &[],
    //         );
    //         pin_mut!(bangumi_metas);
    //         let bangumi_metas = bangumi_metas.try_collect::<Vec<_>>().await?;

    //         assert!(!bangumi_metas.is_empty());

    //         assert!(bangumi_metas[0].origin_poster_src.is_some());

    //         assert!(my_bangumi_with_cookie_mock.matched_async().await);

    //         expand_bangumi_mock.expect(bangumi_metas.len());
    //     }

    //     Ok(())
    // }
}
