use std::ops::Deref;

use bytes::Bytes;
use color_eyre::eyre::{self, ContextCompat};
use html_escape::decode_html_entities;
use itertools::Itertools;
use lazy_static::lazy_static;
use lightningcss::{properties::Property, values::image::Image as CSSImage};
use loco_rs::app::AppContext;
use regex::Regex;
use reqwest::IntoUrl;
use scraper::Html;
use url::Url;

use super::{
    AppMikanClient, MIKAN_BUCKET_KEY, MikanBangumiRssLink, parse_mikan_bangumi_id_from_rss_link,
};
use crate::{
    app::AppContextExt,
    dal::DalContentCategory,
    extract::html::parse_style_attr,
    fetch::{html::fetch_html, image::fetch_image},
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

lazy_static! {
    static ref MIKAN_TITLE_SEASON: Regex = Regex::new("第.*季").unwrap();
}

pub fn build_mikan_bangumi_homepage(
    mikan_base_url: impl IntoUrl,
    mikan_bangumi_id: &str,
    mikan_fansub_id: Option<&str>,
) -> eyre::Result<Url> {
    let mut url = mikan_base_url.into_url()?;
    url.set_path(&format!("/Home/Bangumi/{mikan_bangumi_id}"));
    url.set_fragment(mikan_fansub_id);
    Ok(url)
}

pub fn build_mikan_episode_homepage(
    mikan_base_url: impl IntoUrl,
    mikan_episode_id: &str,
) -> eyre::Result<Url> {
    let mut url = mikan_base_url.into_url()?;
    url.set_path(&format!("/Home/Episode/{mikan_episode_id}"));
    Ok(url)
}

pub fn build_mikan_bangumi_expand_info_url(
    mikan_base_url: impl IntoUrl,
    mikan_bangumi_id: &str,
) -> eyre::Result<Url> {
    let mut url = mikan_base_url.into_url()?;
    url.set_path("/ExpandBangumi");
    url.query_pairs_mut()
        .append_pair("bangumiId", mikan_bangumi_id)
        .append_pair("showSubscribed", "true");
    Ok(url)
}

pub fn parse_mikan_bangumi_id_from_homepage(url: &Url) -> Option<MikanBangumiHomepage> {
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

pub fn parse_mikan_episode_id_from_homepage(url: &Url) -> Option<MikanEpisodeHomepage> {
    if url.path().starts_with("/Home/Episode/") {
        let mikan_episode_id = url.path().replace("/Home/Episode/", "");
        Some(MikanEpisodeHomepage { mikan_episode_id })
    } else {
        None
    }
}

pub async fn parse_mikan_bangumi_poster_from_origin_poster_src(
    client: Option<&AppMikanClient>,
    origin_poster_src_url: Url,
) -> eyre::Result<MikanBangumiPosterMeta> {
    let http_client = client.map(|s| s.deref());
    let poster_data = fetch_image(http_client, origin_poster_src_url.clone()).await?;
    Ok(MikanBangumiPosterMeta {
        origin_poster_src: origin_poster_src_url,
        poster_data: Some(poster_data),
        poster_src: None,
    })
}

pub async fn parse_mikan_bangumi_poster_from_origin_poster_src_with_cache(
    ctx: &AppContext,
    origin_poster_src_url: Url,
    subscriber_id: i32,
) -> eyre::Result<MikanBangumiPosterMeta> {
    let dal_client = ctx.get_dal_client();
    let mikan_client = ctx.get_mikan_client();
    if let Some(poster_src) = dal_client
        .exists_object(
            DalContentCategory::Image,
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

    let poster_data =
        fetch_image(Some(mikan_client.deref()), origin_poster_src_url.clone()).await?;

    let poster_str = dal_client
        .store_object(
            DalContentCategory::Image,
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

pub fn parse_mikan_origin_poster_src_from_style_attr(
    mikan_base_url: impl IntoUrl,
    style_attr: &str,
) -> Option<Url> {
    let base_url = mikan_base_url.into_url().ok()?;
    parse_style_attr(style_attr)
        .and_then(|style| {
            style.iter().find_map(|(prop, _)| {
                match prop {
                    Property::BackgroundImage(images) => {
                        for img in images {
                            if let CSSImage::Url(path) = img {
                                if let Ok(url) = base_url.join(path.url.trim()) {
                                    return Some(url);
                                }
                            }
                        }
                    }
                    Property::Background(backgrounds) => {
                        for bg in backgrounds {
                            if let CSSImage::Url(path) = &bg.image {
                                if let Ok(url) = base_url.join(path.url.trim()) {
                                    return Some(url);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                None
            })
        })
        .map(|mut poster_str| {
            poster_str.set_query(None);
            poster_str.set_fragment(None);
            poster_str
        })
}

pub async fn parse_mikan_bangumi_meta_from_mikan_homepage(
    client: Option<&AppMikanClient>,
    mikan_bangumi_homepage_url: Url,
) -> eyre::Result<MikanBangumiMeta> {
    let http_client = client.map(|s| s.deref());
    let mikan_base_url = mikan_bangumi_homepage_url.origin().unicode_serialization();
    let content = fetch_html(http_client, mikan_bangumi_homepage_url.as_str()).await?;
    let html = Html::parse_document(&content);

    let bangumi_fansubs = html
        .select(&scraper::Selector::parse(".subgroup-text").unwrap())
        .filter_map(|el| {
            if let (Some(fansub_id), Some(fansub_name)) = (
                el.value()
                    .attr("id")
                    .map(|s| decode_html_entities(s).trim().to_string()),
                el.select(&scraper::Selector::parse("a:nth-child(1)").unwrap())
                    .next()
                    .map(|child| {
                        let mut s = String::from(
                            child
                                .prev_sibling()
                                .and_then(|t| t.value().as_text())
                                .map(|s| s.trim())
                                .unwrap_or_default(),
                        );
                        s.extend(child.text());
                        decode_html_entities(&s).trim().to_string()
                    }),
            ) {
                Some((fansub_id, fansub_name))
            } else {
                None
            }
        })
        .collect_vec();

    let fansub_info = mikan_bangumi_homepage_url.fragment().and_then(|b| {
        bangumi_fansubs
            .iter()
            .find_map(|(id, name)| if id == b { Some((id, name)) } else { None })
    });

    let bangumi_title = html
        .select(&scraper::Selector::parse(".bangumi-title").unwrap())
        .next()
        .map(|el| {
            decode_html_entities(&el.text().collect::<String>())
                .trim()
                .to_string()
        })
        .and_then(|title| if title.is_empty() { None } else { Some(title) })
        .wrap_err_with(|| {
            // todo: error handler
            format!(
                "Missing mikan bangumi official title for {}",
                mikan_bangumi_homepage_url
            )
        })?;

    let MikanBangumiRssLink {
        mikan_bangumi_id, ..
    } = html
        .select(&scraper::Selector::parse(".bangumi-title > .mikan-rss").unwrap())
        .next()
        .and_then(|el| el.value().attr("href"))
        .as_ref()
        .and_then(|s| mikan_bangumi_homepage_url.join(s).ok())
        .and_then(|rss_link_url| parse_mikan_bangumi_id_from_rss_link(&rss_link_url))
        .wrap_err_with(|| {
            // todo: error handler
            format!(
                "Missing mikan bangumi rss link or error format for {}",
                mikan_bangumi_homepage_url
            )
        })?;

    let origin_poster_src = html
        .select(&scraper::Selector::parse(".bangumi-poster").unwrap())
        .next()
        .and_then(|el| el.value().attr("style"))
        .and_then(|style_attr| {
            parse_mikan_origin_poster_src_from_style_attr(&mikan_base_url, style_attr)
        });

    Ok(MikanBangumiMeta {
        homepage: mikan_bangumi_homepage_url,
        bangumi_title,
        origin_poster_src,
        mikan_bangumi_id,
        fansub: fansub_info.map(|s| s.1.to_string()),
        mikan_fansub_id: fansub_info.map(|s| s.0.to_string()),
    })
}

pub async fn parse_mikan_episode_meta_from_mikan_homepage(
    client: Option<&AppMikanClient>,
    mikan_episode_homepage_url: Url,
) -> eyre::Result<MikanEpisodeMeta> {
    let http_client = client.map(|s| s.deref());
    let mikan_base_url = mikan_episode_homepage_url.origin().unicode_serialization();
    let content = fetch_html(http_client, mikan_episode_homepage_url.as_str()).await?;

    let html = Html::parse_document(&content);

    let bangumi_title = html
        .select(&scraper::Selector::parse(".bangumi-title").unwrap())
        .next()
        .map(|el| {
            decode_html_entities(&el.text().collect::<String>())
                .trim()
                .to_string()
        })
        .and_then(|title| if title.is_empty() { None } else { Some(title) })
        .wrap_err_with(|| {
            // todo: error handler
            format!(
                "Missing mikan bangumi official title for {}",
                mikan_episode_homepage_url
            )
        })?;

    let episode_title = html
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
        .map(|el| {
            decode_html_entities(&el.text().collect::<String>())
                .replace(" - Mikan Project", "")
                .trim()
                .to_string()
        })
        .and_then(|title| if title.is_empty() { None } else { Some(title) })
        .wrap_err_with(|| {
            // todo: error handler
            format!(
                "Missing mikan episode official title for {}",
                mikan_episode_homepage_url
            )
        })?;

    let (mikan_bangumi_id, mikan_fansub_id) = html
        .select(&scraper::Selector::parse(".bangumi-title > .mikan-rss").unwrap())
        .next()
        .and_then(|el| el.value().attr("href"))
        .as_ref()
        .and_then(|s| mikan_episode_homepage_url.join(s).ok())
        .and_then(|rss_link_url| parse_mikan_bangumi_id_from_rss_link(&rss_link_url))
        .and_then(
            |MikanBangumiRssLink {
                 mikan_bangumi_id,
                 mikan_fansub_id,
                 ..
             }| {
                mikan_fansub_id.map(|mikan_fansub_id| (mikan_bangumi_id, mikan_fansub_id))
            },
        )
        .wrap_err_with(|| {
            // todo: error handler
            format!(
                "Missing mikan bangumi rss link or error format for {}",
                mikan_episode_homepage_url
            )
        })?;

    let fansub = html
        .select(&scraper::Selector::parse(".bangumi-info>.magnet-link-wrap").unwrap())
        .next()
        .map(|el| {
            decode_html_entities(&el.text().collect::<String>())
                .trim()
                .to_string()
        })
        .wrap_err_with(|| {
            // todo: error handler
            format!(
                "Missing mikan bangumi fansub name for {}",
                mikan_episode_homepage_url
            )
        })?;

    let origin_poster_src = html
        .select(&scraper::Selector::parse(".bangumi-poster").unwrap())
        .next()
        .and_then(|el| el.value().attr("style"))
        .and_then(|s| parse_mikan_origin_poster_src_from_style_attr(mikan_base_url, s));

    let MikanEpisodeHomepage {
        mikan_episode_id, ..
    } = parse_mikan_episode_id_from_homepage(&mikan_episode_homepage_url).wrap_err_with(|| {
        format!(
            "Failed to extract mikan_episode_id from {}",
            &mikan_episode_homepage_url
        )
    })?;

    Ok(MikanEpisodeMeta {
        mikan_bangumi_id,
        mikan_fansub_id,
        bangumi_title,
        episode_title,
        homepage: mikan_episode_homepage_url,
        origin_poster_src,
        fansub,
        mikan_episode_id,
    })
}

/**
 * @logined-required
 */
pub async fn parse_mikan_bangumis_meta_from_my_bangumi_page(
    client: Option<&AppMikanClient>,
    my_bangumi_page_url: Url,
) -> eyre::Result<Vec<MikanBangumiMeta>> {
    let http_client = client.map(|c| c.deref());
    let mikan_base_url = my_bangumi_page_url.origin().unicode_serialization();

    let content = fetch_html(http_client, my_bangumi_page_url.clone()).await?;

    let html = Html::parse_document(&content);

    let mut bangumi_list = vec![];
    for bangumi_elem in
        html.select(&scraper::Selector::parse(".sk-bangumi .an-info a.an-text").unwrap())
    {
        if let (Some(bangumi_home_page_url), Some(bangumi_title)) =
            (bangumi_elem.attr("href"), bangumi_elem.attr("title"))
        {
            let origin_poster_src = bangumi_elem
                .prev_sibling()
                .and_then(|ele| ele.value().as_element())
                .and_then(|ele| ele.attr("style"))
                .and_then(|style_attr| {
                    parse_mikan_origin_poster_src_from_style_attr(
                        mikan_base_url.clone(),
                        style_attr,
                    )
                });
            let bangumi_home_page_url = my_bangumi_page_url.join(bangumi_home_page_url)?;
            if let Some(MikanBangumiHomepage {
                ref mikan_bangumi_id,
                ..
            }) = parse_mikan_bangumi_id_from_homepage(&bangumi_home_page_url)
            {
                let bangumi_expand_info_url =
                    build_mikan_bangumi_expand_info_url(mikan_base_url.clone(), mikan_bangumi_id)?;
                let bangumi_expand_info_content =
                    fetch_html(http_client, bangumi_expand_info_url).await?;
                let bangumi_expand_info_fragment =
                    Html::parse_fragment(&bangumi_expand_info_content);
                for fansub_info in bangumi_expand_info_fragment.select(
                    &scraper::Selector::parse("js-expand_bangumi-subgroup.js-subscribed").unwrap(),
                ) {
                    if let (Some(fansub_name), Some(mikan_fansub_id)) = (
                        fansub_info
                            .select(&scraper::Selector::parse(".tag-res-name[title]").unwrap())
                            .next()
                            .and_then(|ele| ele.attr("title")),
                        fansub_info
                            .select(
                                &scraper::Selector::parse(
                                    ".active[data-subtitlegroupid][data-bangumiid]",
                                )
                                .unwrap(),
                            )
                            .next()
                            .and_then(|ele| ele.attr("data-subtitlegroupid")),
                    ) {
                        bangumi_list.push(MikanBangumiMeta {
                            homepage: build_mikan_bangumi_homepage(
                                mikan_base_url.clone(),
                                mikan_bangumi_id.as_str(),
                                Some(mikan_fansub_id),
                            )?,
                            bangumi_title: bangumi_title.to_string(),
                            mikan_bangumi_id: mikan_bangumi_id.to_string(),
                            mikan_fansub_id: Some(mikan_fansub_id.to_string()),
                            fansub: Some(fansub_name.to_string()),
                            origin_poster_src: origin_poster_src.clone(),
                        })
                    }
                }
            }
        }
    }

    Ok(bangumi_list)
}

#[cfg(test)]
mod test {
    use std::assert_matches::assert_matches;

    use color_eyre::eyre;
    use url::Url;
    use zune_image::{codecs::ImageFormat, image::Image};

    use super::{
        parse_mikan_bangumi_meta_from_mikan_homepage,
        parse_mikan_bangumi_poster_from_origin_poster_src,
        parse_mikan_episode_meta_from_mikan_homepage,
    };

    #[tokio::test]
    async fn test_parse_mikan_episode() {
        let test_fn = async || -> eyre::Result<()> {
            let url_str =
                "https://mikanani.me/Home/Episode/475184dce83ea2b82902592a5ac3343f6d54b36a";
            let url = Url::parse(url_str)?;

            let ep_meta = parse_mikan_episode_meta_from_mikan_homepage(None, url.clone()).await?;

            assert_eq!(ep_meta.homepage, url);
            assert_eq!(ep_meta.bangumi_title, "葬送的芙莉莲");
            assert_eq!(
                ep_meta.origin_poster_src,
                Some(Url::parse(
                    "https://mikanani.me/images/Bangumi/202309/5ce9fed1.jpg"
                )?)
            );
            assert_eq!(ep_meta.fansub, "LoliHouse");
            assert_eq!(ep_meta.mikan_fansub_id, "370");
            assert_eq!(ep_meta.mikan_bangumi_id, "3141");

            assert_matches!(ep_meta.origin_poster_src, Some(..));

            let bgm_poster = parse_mikan_bangumi_poster_from_origin_poster_src(
                None,
                ep_meta.origin_poster_src.unwrap(),
            )
            .await?;
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
        };

        test_fn().await.expect("test parse mikan failed");
    }

    #[tokio::test]
    async fn test_parse_mikan_bangumi() {
        let test_fn = async || -> eyre::Result<()> {
            let url_str = "https://mikanani.me/Home/Bangumi/3416#370";
            let url = Url::parse(url_str)?;

            let bgm_meta = parse_mikan_bangumi_meta_from_mikan_homepage(None, url.clone()).await?;

            assert_eq!(bgm_meta.homepage, url);
            assert_eq!(bgm_meta.bangumi_title, "叹气的亡灵想隐退");
            assert_eq!(
                bgm_meta.origin_poster_src,
                Some(Url::parse(
                    "https://mikanani.me/images/Bangumi/202410/480ef127.jpg"
                )?)
            );
            assert_eq!(bgm_meta.fansub, Some(String::from("LoliHouse")));
            assert_eq!(bgm_meta.mikan_fansub_id, Some(String::from("370")));
            assert_eq!(bgm_meta.mikan_bangumi_id, "3416");

            assert_eq!(
                bgm_meta.homepage.as_str(),
                "https://mikanani.me/Home/Bangumi/3416#370"
            );

            Ok(())
        };

        test_fn().await.expect("test parse mikan failed");
    }
}
