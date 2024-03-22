use serde::{Deserialize, Serialize};

use crate::{
    i18n::LanguagePreset,
    models::bangumi::BangumiDistribution,
    parsers::tmdb::{
        tmdb_client::TmdbApiClient,
        tmdb_dtos::{TmdbSearchMultiItemDto, TmdbSearchMultiPageDto},
    },
};

impl BangumiDistribution {
    pub fn prefer_tmdb_media_type(&self) -> &str {
        match self {
            BangumiDistribution::Movie => "movie",
            BangumiDistribution::Tv => "tv",
            _ => "tv",
        }
    }

    pub fn from_tmdb_media_type(media_type: &str) -> Self {
        match media_type {
            "movie" => BangumiDistribution::Movie,
            _ => BangumiDistribution::Tv,
        }
    }
}

const TMDB_ANIMATION_GENRE_ID: i64 = 16;

#[inline]
fn build_tmdb_search_api_url(query: &str, lang: &LanguagePreset, page: u32) -> String {
    format!(
        "{TMDB_API_ORIGIN}/3/search/multi?language={lang_tag}&query={query}&page={page}&\
         include_adult=true",
        lang_tag = lang.name_str(),
    )
}

#[inline]
fn build_tmdb_info_api_url(
    id: i64,
    lang: &LanguagePreset,
    distribution: &BangumiDistribution,
) -> String {
    let tmdb_media_type = match distribution {
        BangumiDistribution::Movie => "movie",
        BangumiDistribution::Tv => "tv",
        _ => "tv",
    };
    format!(
        "{TMDB_API_ORIGIN}/3/{tmdb_media_type}/{id}?language={lang_tag}",
        lang_tag = lang.name_str()
    )
}

fn tmdb_genres_is_match_animation(genre_ids: &[i64]) -> bool {
    genre_ids.contains(&TMDB_ANIMATION_GENRE_ID)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbBangumiItem {
    pub id: i64,
    pub name: String,
    pub origin_name: String,
    pub last_season: i32,
    pub year: Option<String>,
    pub poster_link: Option<String>,
}

pub async fn search_tmdb_items_from_title_and_lang(
    tmdb_client: &TmdbApiClient,
    title: &str,
    lang: &LanguagePreset,
) -> eyre::Result<Vec<TmdbSearchMultiItemDto>> {
    let mut items = vec![];
    let page_num = {
        let search_url = build_tmdb_search_api_url(title, lang, 1);
        let first_page: TmdbSearchMultiPageDto =
            tmdb_client.fetch(|fetch| fetch.get(search_url)).await?;
        items.extend(first_page.results);
        first_page.total_pages
    };
    for i in 2..=page_num {
        let search_url = build_tmdb_search_api_url(title, lang, i);
        let page: TmdbSearchMultiPageDto = tmdb_client.fetch(|fetch| fetch.get(search_url)).await?;
        items.extend(page.results);
    }
    Ok(items)
}

pub async fn get_tmdb_info_from_id_lang_and_distribution(
    tmdb_client: &TmdbApiClient,
    id: i64,
    lang: &LanguagePreset,
    distribution: &BangumiDistribution,
) -> eyre::Result<TmdbSearchMultiItemDto> {
    let info_url = build_tmdb_info_api_url(id, lang, distribution);
    let info: TmdbSearchMultiItemDto = tmdb_client.fetch(|fetch| fetch.get(info_url)).await?;
    Ok(info)
}

pub async fn parse_tmdb_bangumi_from_title_and_lang(
    tmdb_client: &TmdbApiClient,
    title: &str,
    lang: &LanguagePreset,
    distribution: &BangumiDistribution,
) -> eyre::Result<Option<TmdbBangumiItem>> {
    let mut search_result = search_tmdb_items_from_title_and_lang(tmdb_client, title, lang).await?;
    if search_result.is_empty() {
        search_result =
            search_tmdb_items_from_title_and_lang(tmdb_client, &title.replace(' ', ""), lang)
                .await?;
    }
    if search_result.is_empty() {
        return Ok(None);
    } else {
        let mut target_and_priority: Option<(TmdbSearchMultiItemDto, u32)> = None;
        for item in search_result.iter() {
            let is_animation = tmdb_genres_is_match_animation(&item.genre_ids);
            let is_prefer_media_type =
                item.media_type.as_deref() == Some(distribution.prefer_tmdb_media_type());
            let priority =
                (if is_prefer_media_type { 10 } else { 0 }) + (if is_animation { 1 } else { 0 });
            if let Some((last_target_id, last_priority)) = target_and_priority.as_deref_mut() {
                if priority > last_priority {
                    *last_target_id = item;
                }
            } else {
                target_and_priority = Some((item, priority));
            }
        }
        if let Some((target, _)) = target_and_priority {
            let info_url = get_tmdb_info_from_id_lang_and_distribution(
                target.id,
                lang,
                BangumiDistribution::from_tmdb_media_type(target.media_type),
            );
            let info: TmdbSearchMultiItemDto =
                tmdb_client.fetch(|fetch| fetch.get(info_url)).await?;
            let last_season = match distribution {
                BangumiDistribution::Movie => 1,
                BangumiDistribution::Tv => info.number_of_seasons,
                _ => 1,
            };
            Ok(Some(TmdbBangumiItem {
                id: info.id,
                name: info.name,
                origin_name: info.original_name,
                last_season,
                year: info.first_air_date,
                poster_link: info.poster_path,
            }))
        } else {
            Ok(None)
        }
    }
}
