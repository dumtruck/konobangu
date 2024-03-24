use super::tmdb_client::TMDB_API_ORIGIN;
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
        "{endpoint}/4/list/{list_id}?language={lang_tag}&page={page}",
        endpoint = TMDB_API_ORIGIN,
        list_id = list_id,
        lang_tag = lang.name_str(),
        page = page
    )
}

pub async fn parse_tmdb_list_items_from_list_api(
    tmdb_client: &TmdbApiClient,
    list_id: i64,
    lang: &LanguagePreset,
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

#[cfg(test)]
mod tests {
    use super::super::tmdb_client::tests::prepare_tmdb_api_client;

    #[tokio::test]
    async fn test_parse_tmdb_list_items_from_list_api() {
        let client = prepare_tmdb_api_client().await;
        let items = super::parse_tmdb_list_items_from_list_api(
            client.as_ref(),
            8294054,
            &crate::i18n::LanguagePreset::parse("zh-CN").expect("failed to create language preset"),
        )
        .await
        .expect("failed to parse tmdb list items from list api");

        assert!(items.iter().any(|item| item.name == "葬送的芙莉莲"));
    }
}
