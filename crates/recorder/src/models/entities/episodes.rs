//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub raw_name: String,
    pub official_title: String,
    pub display_name: String,
    pub name_zh: Option<String>,
    pub name_jp: Option<String>,
    pub name_en: Option<String>,
    pub s_name_zh: Option<String>,
    pub s_name_jp: Option<String>,
    pub s_name_en: Option<String>,
    pub bangumi_id: i32,
    pub download_id: i32,
    pub save_path: String,
    pub resolution: Option<String>,
    pub season: i32,
    pub season_raw: Option<String>,
    pub fansub: Option<String>,
    pub poster_link: Option<String>,
    pub home_page: Option<String>,
    pub subtitle: Option<Vec<String>>,
    pub deleted: bool,
    pub source: Option<String>,
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
