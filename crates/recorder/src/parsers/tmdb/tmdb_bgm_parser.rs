use serde::{Deserialize, Serialize};

use super::tmdb_client::TMDB_API_ORIGIN;
use crate::{
    i18n::LanguagePreset,
    models::bangumi::BangumiDistribution,
    parsers::tmdb::{
        tmdb_client::TmdbApiClient,
        tmdb_dtos::{
            TmdbMediaDetailDto, TmdbMovieDetailDto, TmdbSearchMultiItemDto, TmdbSearchMultiPageDto,
            TmdbTvSeriesDetailDto,
        },
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
        "{endpoint}/3/search/multi?language={lang_tag}&query={query}&page={page}&\
         include_adult=true",
        endpoint = TMDB_API_ORIGIN,
        lang_tag = lang.name_str(),
        query = query,
        page = page
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
        "{endpoint}/3/{tmdb_media_type}/{id}?language={lang_tag}",
        endpoint = TMDB_API_ORIGIN,
        tmdb_media_type = tmdb_media_type,
        id = id,
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
        let first_page: TmdbSearchMultiPageDto = tmdb_client
            .fetch_json(|fetch| fetch.get(search_url))
            .await?;
        items.extend(first_page.results);
        first_page.total_pages
    };
    for i in 2..=page_num {
        let search_url = build_tmdb_search_api_url(title, lang, i);
        let page: TmdbSearchMultiPageDto = tmdb_client
            .fetch_json(|fetch| fetch.get(search_url))
            .await?;
        items.extend(page.results);
    }
    Ok(items)
}

pub async fn get_tmdb_info_from_id_lang_and_distribution(
    tmdb_client: &TmdbApiClient,
    id: i64,
    lang: &LanguagePreset,
    distribution: &BangumiDistribution,
) -> eyre::Result<TmdbMediaDetailDto> {
    let info_url = build_tmdb_info_api_url(id, lang, distribution);
    let info = if distribution == &BangumiDistribution::Movie {
        let info: Box<TmdbMovieDetailDto> =
            tmdb_client.fetch_json(|fetch| fetch.get(info_url)).await?;
        TmdbMediaDetailDto::Movie(info)
    } else {
        let info: Box<TmdbTvSeriesDetailDto> =
            tmdb_client.fetch_json(|fetch| fetch.get(info_url)).await?;
        TmdbMediaDetailDto::Tv(info)
    };
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
        Ok(None)
    } else {
        let mut target_and_priority: Option<(&TmdbSearchMultiItemDto, u32)> = None;
        for item in search_result.iter() {
            let is_animation = tmdb_genres_is_match_animation(&item.genre_ids);
            let is_prefer_media_type = item.media_type == distribution.prefer_tmdb_media_type();
            let priority =
                (if is_prefer_media_type { 10 } else { 0 }) + (if is_animation { 1 } else { 0 });
            if let Some((last_target, last_priority)) = target_and_priority.as_mut() {
                if priority > *last_priority {
                    *last_target = item;
                }
            } else {
                target_and_priority = Some((item, priority));
            }
        }
        if let Some((target, _)) = target_and_priority {
            let info = get_tmdb_info_from_id_lang_and_distribution(
                tmdb_client,
                target.id,
                lang,
                &BangumiDistribution::from_tmdb_media_type(&target.media_type),
            )
            .await?;
            match info {
                TmdbMediaDetailDto::Movie(info) => Ok(Some(TmdbBangumiItem {
                    id: info.id,
                    name: info.name,
                    origin_name: info.original_name,
                    last_season: 1,
                    year: Some(info.release_date),
                    poster_link: info.poster_path,
                })),
                TmdbMediaDetailDto::Tv(info) => Ok(Some(TmdbBangumiItem {
                    id: info.id,
                    name: info.name,
                    origin_name: info.original_name,
                    last_season: info.number_of_seasons,
                    year: info.first_air_date,
                    poster_link: info.poster_path,
                })),
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::tmdb::{
        tmdb_bgm_parser::parse_tmdb_bangumi_from_title_and_lang,
        tmdb_client::tests::prepare_tmdb_api_client,
    };

    #[tokio::test]
    async fn test_parse_tmdb_bangumi_from_title_and_lang() {
        let client = prepare_tmdb_api_client().await;
        let result = parse_tmdb_bangumi_from_title_and_lang(
            client.as_ref(),
            "青春猪头",
            &crate::i18n::LanguagePreset::parse("zh-CN").expect("failed to create language preset"),
            &crate::models::bangumi::BangumiDistribution::Tv,
        )
        .await
        .expect("failed to parse tmdb bangumi from title and lang");

        assert_eq!(
            result.as_ref().map_or("", |item| &item.name),
            "青春猪头少年不会梦到兔女郎学姐"
        );
    }
}
