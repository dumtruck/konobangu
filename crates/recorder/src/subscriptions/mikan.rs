use crate::rss::engine::RssEngine;

pub struct MikanRssCreateDto {
    pub rss_link: String,
    pub display_name: String,
    pub aggregate: bool,
    pub enabled: Option<bool>,
}

pub struct MikanSubscriptionEngine {
}

impl MikanSubscriptionEngine {
    pub async fn add_rss(create_dto: MikanRssCreateDto) -> eyre::Result<()> {
        let content = reqwest::get(&create_dto.rss_link).await?.bytes().await?;
        let channel = rss::Channel::read_from(&content[..])?;

        Ok(())
    }
}

pub struct MikanSubscriptionItem {
}
