use sea_orm::{prelude::*, ActiveValue};

use crate::extract::mikan::MikanRssItem;
pub use crate::models::entities::downloads::*;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_mikan_rss_item(m: MikanRssItem, subscription_id: i32) -> Self {
        let _ = Self {
            origin_name: ActiveValue::Set(m.title.clone()),
            display_name: ActiveValue::Set(m.title),
            subscription_id: ActiveValue::Set(subscription_id),
            status: ActiveValue::Set(DownloadStatus::Pending),
            mime: ActiveValue::Set(DownloadMime::BitTorrent),
            url: ActiveValue::Set(m.url.to_string()),
            curr_size: ActiveValue::Set(m.content_length.as_ref().map(|_| 0)),
            all_size: ActiveValue::Set(m.content_length),
            homepage: ActiveValue::Set(Some(m.homepage.to_string())),
            ..Default::default()
        };
        todo!()
    }
}

impl Model {}
