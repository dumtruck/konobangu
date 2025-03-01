use std::borrow::Cow;

use async_stream::try_stream;
use bytes::Bytes;
use futures::Stream;
use itertools::Itertools;
use scraper::{Html, Selector};
use tracing::instrument;
use url::Url;

use super::{
    MIKAN_BUCKET_KEY, MikanBangumiRssLink, MikanClient, extract_mikan_bangumi_id_from_rss_link,
};
use crate::{
    app::AppContext,
    errors::{RError, RResult},
    extract::{
        html::{extract_background_image_src_from_style_attr, extract_inner_text_from_element_ref},
        media::extract_image_src_from_str,
    },
    fetch::{html::fetch_html, image::fetch_image},
    storage::StorageContentCategory,
};

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

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpisodeHomepage {
    pub mikan_episode_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanBangumiHomepage {
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: Option<String>,
}

pub fn build_mikan_bangumi_homepage(
    mikan_base_url: Url,
    mikan_bangumi_id: &str,
    mikan_fansub_id: Option<&str>,
) -> Url {
    let mut url = mikan_base_url;
    url.set_path(&format!("/Home/Bangumi/{mikan_bangumi_id}"));
    url.set_fragment(mikan_fansub_id);
    url
}

pub fn build_mikan_episode_homepage(mikan_base_url: Url, mikan_episode_id: &str) -> Url {
    let mut url = mikan_base_url;
    url.set_path(&format!("/Home/Episode/{mikan_episode_id}"));
    url
}

pub fn build_mikan_bangumi_expand_info_url(mikan_base_url: Url, mikan_bangumi_id: &str) -> Url {
    let mut url = mikan_base_url;
    url.set_path("/ExpandBangumi");
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id)
        .append_pair("showSubscribed", "true");
    url
}

pub fn extract_mikan_bangumi_id_from_homepage(url: &Url) -> Option<MikanBangumiHomepage> {
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

pub fn extract_mikan_episode_id_from_homepage(url: &Url) -> Option<MikanEpisodeHomepage> {
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
) -> Result<MikanBangumiPosterMeta, RError> {
    let poster_data = fetch_image(http_client, origin_poster_src_url.clone()).await?;
    Ok(MikanBangumiPosterMeta {
        origin_poster_src: origin_poster_src_url,
        poster_data: Some(poster_data),
        poster_src: None,
    })
}

pub async fn extract_mikan_bangumi_poster_meta_from_src_with_cache(
    ctx: &AppContext,
    origin_poster_src_url: Url,
    subscriber_id: i32,
) -> RResult<MikanBangumiPosterMeta> {
    let dal_client = &ctx.storage;
    let mikan_client = &ctx.mikan;
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
) -> Result<MikanEpisodeMeta, RError> {
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
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title")))
        .inspect_err(|error| {
            tracing::warn!(error = %error);
        })?;

    let MikanBangumiRssLink {
        mikan_bangumi_id,
        mikan_fansub_id,
        ..
    } = html
        .select(mikan_bangumi_id_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(|s| mikan_episode_homepage_url.join(s).ok())
        .and_then(|rss_link_url| extract_mikan_bangumi_id_from_rss_link(&rss_link_url))
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_bangumi_id")))
        .inspect_err(|error| tracing::error!(error = %error))?;

    let mikan_fansub_id = mikan_fansub_id
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_fansub_id")))
        .inspect_err(|error| tracing::error!(error = %error))?;

    let episode_title = html
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(extract_inner_text_from_element_ref)
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("episode_title")))
        .inspect_err(|error| {
            tracing::warn!(error = %error);
        })?;

    let MikanEpisodeHomepage {
        mikan_episode_id, ..
    } = extract_mikan_episode_id_from_homepage(&mikan_episode_homepage_url)
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_episode_id")))
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
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("fansub_name")))
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
) -> Result<MikanBangumiMeta, RError> {
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
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("bangumi_title")))
        .inspect_err(|error| tracing::warn!(error = %error))?;

    let mikan_bangumi_id = html
        .select(mikan_bangumi_id_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(|s| mikan_bangumi_homepage_url.join(s).ok())
        .and_then(|rss_link_url| extract_mikan_bangumi_id_from_rss_link(&rss_link_url))
        .map(
            |MikanBangumiRssLink {
                 mikan_bangumi_id, ..
             }| mikan_bangumi_id,
        )
        .ok_or_else(|| RError::from_mikan_meta_missing_field(Cow::Borrowed("mikan_bangumi_id")))
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

    let (mikan_fansub_id, fansub_name) = mikan_bangumi_homepage_url
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
        fansub_name,
        mikan_fansub_id,
        "mikan bangumi meta extracted"
    );

    Ok(MikanBangumiMeta {
        homepage: mikan_bangumi_homepage_url,
        bangumi_title,
        origin_poster_src,
        mikan_bangumi_id,
        fansub: fansub_name,
        mikan_fansub_id,
    })
}

/**
 * @logined-required
 */
#[instrument(skip_all, fields(my_bangumi_page_url = my_bangumi_page_url.as_str()))]
pub fn extract_mikan_bangumis_meta_from_my_bangumi_page(
    http_client: &MikanClient,
    my_bangumi_page_url: Url,
) -> impl Stream<Item = Result<MikanBangumiMeta, RError>> {
    try_stream! {
        let mikan_base_url = Url::parse(&my_bangumi_page_url.origin().unicode_serialization())?;

        let content = fetch_html(http_client, my_bangumi_page_url.clone()).await?;

        let bangumi_container_selector = &Selector::parse(".sk-bangumi .an-ul>li").unwrap();
        let bangumi_info_selector = &Selector::parse(".an-info a.an-text").unwrap();
        let bangumi_poster_selector =
            &Selector::parse("span[data-src][data-bangumiid], span[data-bangumiid][style]")
                .unwrap();
        let fansub_container_selector =
        &Selector::parse(".js-expand_bangumi-subgroup.js-subscribed").unwrap();
        let fansub_title_selector = &Selector::parse(".tag-res-name[title]").unwrap();
        let fansub_id_selector =
            &Selector::parse(".active[data-subtitlegroupid][data-bangumiid]").unwrap();

        let bangumi_iters = {
            let html = Html::parse_document(&content);

            html.select(bangumi_container_selector)
                .filter_map(|bangumi_elem| {
                    let title_and_href_elem = bangumi_elem.select(bangumi_info_selector).next();
                    let poster_elem = bangumi_elem.select(bangumi_poster_selector).next();
                    if let (Some(bangumi_home_page_url), Some(bangumi_title)) = (
                        title_and_href_elem.and_then(|elem| elem.attr("href")),
                        title_and_href_elem.and_then(|elem| elem.attr("title")),
                    ) {
                        let origin_poster_src = poster_elem.and_then(|ele| {
                            ele.attr("data-src")
                                .and_then(|data_src| {
                                    extract_image_src_from_str(data_src, &mikan_base_url)
                                })
                                .or_else(|| {
                                    ele.attr("style").and_then(|style| {
                                        extract_background_image_src_from_style_attr(
                                            style,
                                            &mikan_base_url,
                                        )
                                    })
                                })
                        });
                        let bangumi_title = bangumi_title.to_string();
                        let bangumi_home_page_url =
                            my_bangumi_page_url.join(bangumi_home_page_url).ok()?;
                        let MikanBangumiHomepage {
                            mikan_bangumi_id, ..
                        } = extract_mikan_bangumi_id_from_homepage(&bangumi_home_page_url)?;
                        if let Some(origin_poster_src) = origin_poster_src.as_ref() {
                            tracing::trace!(
                                origin_poster_src = origin_poster_src.as_str(),
                                bangumi_title,
                                mikan_bangumi_id,
                                "bangumi info extracted"
                            );
                        } else {
                            tracing::warn!(
                                bangumi_title,
                                mikan_bangumi_id,
                                "bangumi info extracted, but failed to extract poster_src"
                            );
                        }
                        let bangumi_expand_info_url = build_mikan_bangumi_expand_info_url(
                            mikan_base_url.clone(),
                            &mikan_bangumi_id,
                        );
                        Some((
                            bangumi_title,
                            mikan_bangumi_id,
                            bangumi_expand_info_url,
                            origin_poster_src,
                        ))
                    } else {
                        None
                    }
                })
                .collect_vec()
        };

        for (bangumi_title, mikan_bangumi_id, bangumi_expand_info_url, origin_poster_src) in
        bangumi_iters
        {
            if let Some((fansub_name, mikan_fansub_id)) = {
                let bangumi_expand_info_content = fetch_html(http_client, bangumi_expand_info_url).await?;
                let bangumi_expand_info_fragment = Html::parse_fragment(&bangumi_expand_info_content);
                bangumi_expand_info_fragment.select(fansub_container_selector).next().and_then(|fansub_info| {
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
                            .map(String::from)
                    ) {
                        Some((fansub_name, mikan_fansub_id))
                    } else {
                        None
                    }
                })
            } {
                tracing::trace!(
                    fansub_name,
                    mikan_fansub_id,
                    "subscribed fansub extracted"
                );
                yield MikanBangumiMeta {
                    homepage: build_mikan_bangumi_homepage(
                        mikan_base_url.clone(),
                        &mikan_bangumi_id,
                        Some(&mikan_fansub_id),
                    ),
                    bangumi_title: bangumi_title.to_string(),
                    mikan_bangumi_id: mikan_bangumi_id.to_string(),
                    mikan_fansub_id: Some(mikan_fansub_id),
                    fansub: Some(fansub_name),
                    origin_poster_src: origin_poster_src.clone(),
                };
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(unused_variables)]
    use color_eyre::eyre;
    use futures::{TryStreamExt, pin_mut};
    use http::header;
    use rstest::{fixture, rstest};
    use secrecy::SecretString;
    use tracing::Level;
    use url::Url;
    use zune_image::{codecs::ImageFormat, image::Image};

    use super::*;
    use crate::{
        extract::mikan::{
            MikanAuthSecrecy, web_extract::extract_mikan_bangumis_meta_from_my_bangumi_page,
        },
        test_utils::{mikan::build_testing_mikan_client, tracing::init_testing_tracing},
    };

    #[fixture]
    fn before_each() {
        init_testing_tracing(Level::INFO);
    }

    #[rstest]
    #[tokio::test]
    async fn test_extract_mikan_poster_from_src(before_each: ()) -> eyre::Result<()> {
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
    async fn test_extract_mikan_episode(before_each: ()) -> eyre::Result<()> {
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
    ) -> eyre::Result<()> {
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
    #[tokio::test]
    async fn test_extract_mikan_bangumis_meta_from_my_bangumi_page(
        before_each: (),
    ) -> eyre::Result<()> {
        let mut mikan_server = mockito::Server::new_async().await;

        let mikan_base_url = Url::parse(&mikan_server.url())?;

        let my_bangumi_page_url = mikan_base_url.join("/Home/MyBangumi")?;

        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        {
            let my_bangumi_without_cookie_mock = mikan_server
                .mock("GET", my_bangumi_page_url.path())
                .match_header(header::COOKIE, mockito::Matcher::Missing)
                .with_body_from_file("tests/resources/mikan/MyBangumi-noauth.htm")
                .create_async()
                .await;

            let bangumi_metas = extract_mikan_bangumis_meta_from_my_bangumi_page(
                &mikan_client,
                my_bangumi_page_url.clone(),
            );

            pin_mut!(bangumi_metas);

            let bangumi_metas = bangumi_metas.try_collect::<Vec<_>>().await?;

            assert!(bangumi_metas.is_empty());

            assert!(my_bangumi_without_cookie_mock.matched_async().await);
        }
        {
            let my_bangumi_with_cookie_mock = mikan_server
                .mock("GET", my_bangumi_page_url.path())
                .match_header(
                    header::COOKIE,
                    mockito::Matcher::AllOf(vec![
                        mockito::Matcher::Regex(String::from(".*\\.AspNetCore\\.Antiforgery.*")),
                        mockito::Matcher::Regex(String::from(
                            ".*\\.AspNetCore\\.Identity\\.Application.*",
                        )),
                    ]),
                )
                .with_body_from_file("tests/resources/mikan/MyBangumi.htm")
                .create_async()
                .await;

            let expand_bangumi_mock = mikan_server
                .mock("GET", "/ExpandBangumi")
                .match_query(mockito::Matcher::Any)
                .with_body_from_file("tests/resources/mikan/ExpandBangumi.htm")
                .create_async()
                .await;

            let mikan_client_with_cookie = mikan_client.fork_with_auth(MikanAuthSecrecy {
                cookie: SecretString::from(
                    "mikan-announcement=1; .AspNetCore.Antiforgery.abc=abc;  \
                     .AspNetCore.Identity.Application=abc; ",
                ),
                user_agent: Some(String::from(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like \
                     Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0",
                )),
            })?;

            let bangumi_metas = extract_mikan_bangumis_meta_from_my_bangumi_page(
                &mikan_client_with_cookie,
                my_bangumi_page_url,
            );
            pin_mut!(bangumi_metas);
            let bangumi_metas = bangumi_metas.try_collect::<Vec<_>>().await?;

            assert!(!bangumi_metas.is_empty());

            assert!(bangumi_metas[0].origin_poster_src.is_some());

            assert!(my_bangumi_with_cookie_mock.matched_async().await);

            expand_bangumi_mock.expect(bangumi_metas.len());
        }

        Ok(())
    }
}
