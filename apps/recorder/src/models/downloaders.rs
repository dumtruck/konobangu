use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "downloader_type")]
#[serde(rename_all = "snake_case")]
pub enum DownloaderCategory {
    #[sea_orm(string_value = "qbittorrent")]
    QBittorrent,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "downloaders")]
pub struct Model {
    #[sea_orm(column_type = "Timestamp")]
    pub created_at: DateTime,
    #[sea_orm(column_type = "Timestamp")]
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub category: DownloaderCategory,
    pub endpoint: String,
    pub password: String,
    pub username: String,
    pub subscriber_id: i32,
    pub save_path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Subscriber,
    #[sea_orm(has_many = "super::downloads::Entity")]
    Download,
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::downloads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Download.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub fn endpoint_url(&self) -> Result<Url, url::ParseError> {
        let url = Url::parse(&self.endpoint)?;
        Ok(url)
    }
}
