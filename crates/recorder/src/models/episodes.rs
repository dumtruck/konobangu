use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

use crate::{
    models::resources,
    parsers::{mikan::MikanEpisodeMeta, raw::RawEpisodeMeta},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub origin_title: String,
    pub official_title: String,
    pub display_name: String,
    pub name_zh: Option<String>,
    pub name_jp: Option<String>,
    pub name_en: Option<String>,
    pub s_name_zh: Option<String>,
    pub s_name_jp: Option<String>,
    pub s_name_en: Option<String>,
    pub bangumi_id: i32,
    pub resource_id: Option<i32>,
    pub save_path: Option<String>,
    pub resolution: Option<String>,
    pub season: i32,
    pub season_raw: Option<String>,
    pub fansub: Option<String>,
    pub poster_link: Option<String>,
    pub homepage: Option<String>,
    pub subtitle: Option<Vec<String>>,
    pub source: Option<String>,
    pub ep_index: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bangumi::Entity",
        from = "Column::BangumiId",
        to = "super::bangumi::Column::Id"
    )]
    Bangumi,
    #[sea_orm(
        belongs_to = "super::resources::Entity",
        from = "Column::ResourceId",
        to = "super::resources::Column::Id"
    )]
    Resources,
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

impl Related<super::resources::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Resources.def()
    }
}

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
            homepage: ActiveValue::Set(resource.homepage),
            subtitle: ActiveValue::Set(raw_meta.sub),
            source: ActiveValue::Set(raw_meta.source),
            ep_index: ActiveValue::Set(raw_meta.episode_index),
            ..Default::default()
        }
    }
}
