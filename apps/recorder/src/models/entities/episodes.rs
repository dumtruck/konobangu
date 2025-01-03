//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2
use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, Default)]
pub struct EpisodeExtra {
    pub name_zh: Option<String>,
    pub s_name_zh: Option<String>,
    pub name_en: Option<String>,
    pub s_name_en: Option<String>,
    pub name_jp: Option<String>,
    pub s_name_jp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub mikan_episode_id: Option<String>,
    pub raw_name: String,
    pub display_name: String,
    pub bangumi_id: i32,
    pub subscription_id: i32,
    pub subscriber_id: i32,
    pub download_id: Option<i32>,
    pub save_path: Option<String>,
    pub resolution: Option<String>,
    pub season: i32,
    pub season_raw: Option<String>,
    pub fansub: Option<String>,
    pub poster_link: Option<String>,
    pub episode_index: i32,
    pub homepage: Option<String>,
    pub subtitle: Option<Vec<String>>,
    #[sea_orm(default = "false")]
    pub deleted: bool,
    pub source: Option<String>,
    pub extra: EpisodeExtra,
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
        belongs_to = "super::downloads::Entity",
        from = "Column::DownloadId",
        to = "super::downloads::Column::Id"
    )]
    Downloads,
    #[sea_orm(
        belongs_to = "super::subscriptions::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscriptions::Column::Id"
    )]
    Subscriptions,
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id"
    )]
    Subscriber,
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

impl Related<super::downloads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Downloads.def()
    }
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriptions.def()
    }
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}
