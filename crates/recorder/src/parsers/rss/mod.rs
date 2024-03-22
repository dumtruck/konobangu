use crate::{
    models::entities::subscriptions,
    parsers::mikan::{parse_episode_meta_from_mikan_homepage, MikanRssItem},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RssItem {
    Mikan(MikanRssItem),
}

// pub async fn parse_official_title_from_rss_item (rss: &subscriptions::Model)
// -> String {     if rss.category == subscriptions::SubscriptionCategory::Mikan
// {         let res = parse_episode_meta_from_mikan_homepage(rss.source_url)
//     }
// }
