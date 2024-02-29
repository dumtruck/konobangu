use bytes::Bytes;
use html_escape::decode_html_entities;
use lazy_static::lazy_static;
use lightningcss::{properties::Property, values::image::Image};
use regex::Regex;
use url::Url;

use crate::{
    downloaders::{html::download_html, image::download_image},
    parsers::html_parser::{get_tag_style, query_selector_first_tag},
};

pub struct MikanEpisodeMeta {
    pub homepage: Url,
    pub poster_src: Option<Url>,
    pub poster_data: Option<Bytes>,
    pub official_title: String,
}

lazy_static! {
    pub static ref MIKAN_TITLE_SEASON: Regex = Regex::new("第.*季").unwrap();
}

pub async fn parse_episode_meta_from_mikan_homepage(
    url: Url,
) -> eyre::Result<Option<MikanEpisodeMeta>> {
    let url_host = url.origin().unicode_serialization();
    let content = download_html(url.as_str()).await?;
    let dom = tl::parse(&content, tl::ParserOptions::default())?;
    let parser = dom.parser();
    let poster_node = query_selector_first_tag(&dom, r"div.bangumi-poster", parser);
    let official_title_node = query_selector_first_tag(&dom, r"p.bangumi-title", parser);
    let mut poster_src = None;
    if let Some(style) = poster_node.and_then(get_tag_style) {
        for (prop, _) in style.iter() {
            match prop {
                Property::BackgroundImage(images) => {
                    if let Some(Image::Url(path)) = images.first() {
                        if let Ok(url) = Url::parse(&url_host).and_then(|s| s.join(path.url.trim()))
                        {
                            poster_src = Some(url);
                        }
                    }
                }
                Property::Background(backgrounds) => {
                    for bg in backgrounds {
                        if let Image::Url(path) = &bg.image {
                            if let Ok(url) =
                                Url::parse(&url_host).and_then(|s| s.join(path.url.trim()))
                            {
                                poster_src = Some(url);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    };
    poster_src = poster_src.map(|mut p| {
        p.set_query(None);
        p
    });
    let poster_data = if let Some(p) = poster_src.as_ref() {
        download_image(p.as_str()).await.ok()
    } else {
        None
    };
    let meta = official_title_node
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
        .map(|title| MikanEpisodeMeta {
            homepage: url,
            poster_src,
            official_title: title,
            poster_data,
        });
    Ok(meta)
}

#[cfg(test)]
mod test {
    use url::Url;

    use crate::parsers::mikan_ep_parser::parse_episode_meta_from_mikan_homepage;

    #[tokio::test]
    async fn test_parse_mikan() {
        let test_fn = async || -> eyre::Result<()> {
            let url_str =
                "https://mikanani.me/Home/Episode/475184dce83ea2b82902592a5ac3343f6d54b36a";
            let url = Url::parse(url_str)?;

            if let Some(ep_meta) = parse_episode_meta_from_mikan_homepage(url.clone()).await? {
                assert_eq!(ep_meta.homepage, url);
                assert_eq!(
                    ep_meta.poster_src,
                    Some(Url::parse(
                        "https://mikanani.me/images/Bangumi/202309/5ce9fed1.jpg"
                    )?)
                );
                assert_eq!(ep_meta.official_title, "葬送的芙莉莲");
                let u8_data = ep_meta.poster_data.expect("should have poster data");
                assert!(
                    u8_data.starts_with(&[255, 216, 255, 224]),
                    "should start with valid jpeg data magic number"
                );
            } else {
                panic!("can not find mikan episode title")
            }

            Ok(())
        };

        test_fn().await.expect("test parse mikan failed");
    }
}
