use std::fmt::Debug;

use crate::{
    i18n::LanguagePreset,
    parsers::tmdb::{
        tmdb_client::TmdbApiClient,
        tmdb_dtos::{TmdbListItemDto, TmdbListPageDto},
    },
};

#[inline]
fn build_tmdb_list_api_url(list_id: i64, lang: &LanguagePreset, page: u32) -> String {
    format!(
        "{TMDB_API_ORIGIN}/4/list/{list_id}?language={lang_tag}&{page}",
        lang_tag = lang.name_str()
    )
}

pub async fn parse_tmdb_list_items_from_list_api(
    list_id: i64,
    lang: &LanguagePreset,
    tmdb_client: &TmdbApiClient,
) -> eyre::Result<Vec<TmdbListItemDto>> {
    let mut items: Vec<TmdbListItemDto> = vec![];

    let page_num = {
        let first_page: TmdbListPageDto = tmdb_client
            .fetch(|fetch| fetch.get(build_tmdb_list_api_url(list_id, lang, 1)))
            .await?;

        items.extend(first_page.results);

        first_page.total_pages
    };

    for i in 2..=page_num {
        let page: TmdbListPageDto = tmdb_client
            .fetch(|fetch| fetch.get(build_tmdb_list_api_url(list_id, lang, i)))
            .await?;
        items.extend(page.results);
    }

    Ok(items)
}
