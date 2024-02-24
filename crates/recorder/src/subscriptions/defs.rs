use crate::models::prelude::*;

pub struct RssCreateDto {
    pub rss_link: String,
    pub display_name: String,
    pub aggregate: bool,
    pub category: SubscriptionCategory,
    pub enabled: Option<bool>,
}