use bytes::Bytes;
use html_escape::decode_html_entities;
use lazy_static::lazy_static;
use lightningcss::{properties::Property, values::image::Image};
use regex::Regex;
use reqwest::IntoUrl;
use tracing::instrument;
use url::Url;

use crate::parsers::{
    errors::ParseError,
    html::{get_tag_style, query_selector_first_tag},
    mikan::mikan_client::MikanClient,
};

#[derive(Clone, Debug)]
pub struct MikanEpisodeMetaPosterBlob {
    pub origin_url: Url,
    pub data: Bytes,
}

#[derive(Clone, Debug)]
pub struct MikanEpisodeMeta {
    pub homepage: Url,
    pub poster_url: Option<Url>,
    pub official_title: String,
}

lazy_static! {
    static ref MIKAN_TITLE_SEASON: Regex = Regex::new("第.*季").unwrap();
}

#[instrument(skip(client, url))]
pub async fn parse_episode_meta_from_mikan_homepage(
    client: &MikanClient,
    url: impl IntoUrl,
) -> eyre::Result<MikanEpisodeMeta> {
    let url = url.into_url()?;
    let url_host = url.origin().unicode_serialization();
    let content = client.fetch_text(|f| f.get(url.clone())).await?;
    let dom = tl::parse(&content, tl::ParserOptions::default())?;
    let parser = dom.parser();
    let poster_node = query_selector_first_tag(&dom, r"div.bangumi-poster", parser);
    let official_title_node = query_selector_first_tag(&dom, r"p.bangumi-title", parser);
    let mut origin_poster_src = None;
    if let Some(style) = poster_node.and_then(get_tag_style) {
        for (prop, _) in style.iter() {
            match prop {
                Property::BackgroundImage(images) => {
                    if let Some(Image::Url(path)) = images.first() {
                        if let Ok(url) = Url::parse(&url_host).and_then(|s| s.join(path.url.trim()))
                        {
                            origin_poster_src = Some(url);
                        }
                    }
                }
                Property::Background(backgrounds) => {
                    for bg in backgrounds {
                        if let Image::Url(path) = &bg.image {
                            if let Ok(url) =
                                Url::parse(&url_host).and_then(|s| s.join(path.url.trim()))
                            {
                                origin_poster_src = Some(url);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    };
    origin_poster_src = origin_poster_src.map(|mut p| {
        p.set_query(None);
        p
    });
    let official_title = official_title_node
        .map(|s| s.inner_text(parser))
        .and_then(|official_title| {
            let title = MIKAN_TITLE_SEASON
                .replace(&decode_html_entities(&official_title), "")
                .trim()
                .to_string();
            if title.is_empty() {
                None
            } else {
                Some(title)
            }
        })
        .ok_or_else(|| ParseError::MikanEpisodeMetaEmptyOfficialTitleError(url.to_string()))?;

    Ok(MikanEpisodeMeta {
        homepage: url,
        poster_url: origin_poster_src,
        official_title,
    })
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::parse_episode_meta_from_mikan_homepage;
    use crate::parsers::mikan::mikan_client::MikanClient;

    #[tokio::test]
    async fn test_parse_mikan() {
        let test_fn = async || -> eyre::Result<()> {
            let url_str =
                "https://mikanani.me/Home/Episode/475184dce83ea2b82902592a5ac3343f6d54b36a";
            let url = Url::parse(url_str)?;

            let client = MikanClient::new(0).await.expect("should get mikan client");

            let ep_meta = parse_episode_meta_from_mikan_homepage(&client, url.clone()).await?;
            {
                assert_eq!(ep_meta.homepage, url);
                assert_eq!(ep_meta.official_title, "葬送的芙莉莲");
                assert_eq!(
                    ep_meta.poster_url.clone(),
                    Some(Url::parse(
                        "https://mikanani.me/images/Bangumi/202309/5ce9fed1.jpg"
                    )?)
                );
            }

            Ok(())
        };

        test_fn().await.expect("test parse mikan failed");
    }
}
