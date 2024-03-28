use sea_orm::{entity::prelude::*, ActiveValue};

pub use super::entities::episodes::*;
use crate::{
    models::resources,
    parsers::{mikan::MikanEpisodeMeta, raw::RawEpisodeMeta},
};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_mikan_meta(
        bangumi_id: i32,
        resource: resources::Model,
        raw_meta: RawEpisodeMeta,
        mikan_meta: MikanEpisodeMeta,
        mikan_poster: Option<String>,
    ) -> Self {
        Self {
            origin_title: ActiveValue::Set(resource.origin_title),
            official_title: ActiveValue::Set(mikan_meta.official_title.clone()),
            display_name: ActiveValue::Set(mikan_meta.official_title),
            name_zh: ActiveValue::Set(raw_meta.name_zh),
            name_jp: ActiveValue::Set(raw_meta.name_jp),
            name_en: ActiveValue::Set(raw_meta.name_en),
            s_name_zh: ActiveValue::Set(raw_meta.s_name_zh),
            s_name_jp: ActiveValue::Set(raw_meta.s_name_jp),
            s_name_en: ActiveValue::Set(raw_meta.s_name_en),
            bangumi_id: ActiveValue::Set(bangumi_id),
            resource_id: ActiveValue::Set(Some(resource.id)),
            resolution: ActiveValue::Set(raw_meta.resolution),
            season: ActiveValue::Set(raw_meta.season),
            season_raw: ActiveValue::Set(raw_meta.season_raw),
            fansub: ActiveValue::Set(raw_meta.fansub),
            poster_link: ActiveValue::Set(mikan_poster),
            home_page: ActiveValue::Set(resource.homepage),
            subtitle: ActiveValue::Set(raw_meta.sub),
            source: ActiveValue::Set(raw_meta.source),
            ..Default::default()
        }
    }
}
