use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "download_status")]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "downloading")]
    Downloading,
    #[sea_orm(string_value = "paused")]
    Paused,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[derive(
Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "download_mime")]
pub enum DownloadMime {
    #[sea_orm(string_value = "application/octet-stream")]
    #[serde(rename = "application/octet-stream")]
    OctetStream,
    #[sea_orm(string_value = "application/x-bittorrent")]
    #[serde(rename = "application/x-bittorrent")]
    BitTorrent,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "downloads")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub origin_name: String,
    pub display_name: String,
    pub subscription_id: i32,
    pub status: DownloadStatus,
    pub mime: DownloadMime,
    pub url: String,
    pub all_size: u64,
    pub curr_size: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::subscriptions::Entity",
    from = "Column::SubscriptionId",
    to = "super::subscriptions::Column::Id"
    )]
    Subscription,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episode,
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Episode.def()
    }
}
