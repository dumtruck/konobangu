use sea_orm::{entity::prelude::*, ActiveValue};

pub use super::entities::episodes::*;
use crate::models::{bangumi, downloads};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub async fn from_mikan_rss_item(dl: &downloads::Model, bgm: &bangumi::Model) -> Self {
        let _ = Self {
            raw_name: ActiveValue::Set(dl.origin_name.clone()),
            official_title: ActiveValue::Set(bgm.official_title.clone()),
            display_name: ActiveValue::Set(bgm.display_name.clone()),
            name_zh: Default::default(),
            name_jp: Default::default(),
            name_en: Default::default(),
            s_name_zh: Default::default(),
            s_name_jp: Default::default(),
            s_name_en: Default::default(),
            bangumi_id: Default::default(),
            download_id: Default::default(),
            save_path: Default::default(),
            resolution: Default::default(),
            season: Default::default(),
            season_raw: Default::default(),
            fansub: Default::default(),
            poster_link: Default::default(),
            home_page: Default::default(),
            subtitle: Default::default(),
            deleted: Default::default(),
            source: Default::default(),
            ..Default::default()
        };
        todo!()
    }
}
