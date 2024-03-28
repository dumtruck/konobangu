use sea_orm::{prelude::*, ActiveValue};

pub use crate::models::entities::downloads::*;
use crate::parsers::mikan::MikanRssItem;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_mikan_rss_item(rss_item: MikanRssItem, subscription_id: i32) -> Self {
        let download_mime = rss_item.get_download_mime();
        Self {
            origin_title: ActiveValue::Set(rss_item.title.clone()),
            display_name: ActiveValue::Set(rss_item.title),
            subscription_id: ActiveValue::Set(subscription_id),
            status: ActiveValue::Set(DownloadStatus::Pending),
            mime: ActiveValue::Set(download_mime),
            url: ActiveValue::Set(rss_item.url),
            all_size: ActiveValue::Set(rss_item.content_length),
            curr_size: ActiveValue::Set(Some(0)),
            homepage: ActiveValue::Set(rss_item.homepage),
            ..Default::default()
        }
    }
}
