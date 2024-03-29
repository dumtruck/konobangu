use std::future::Future;

use bytes::Bytes;
use loco_rs::app::AppContext;
use sea_orm::{entity::prelude::*, ActiveValue, TryIntoModel};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    parsers::{errors::ParseError, mikan::MikanRssItem},
    path::extract_extname_from_url,
    storage::{AppContextDalExt, DalContentType},
};

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
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "resource_category")]
pub enum ResourceCategory {
    #[sea_orm(string_value = "octet-stream")]
    #[serde(rename = "octet-stream")]
    OctetStream,
    #[sea_orm(string_value = "bittorrent")]
    #[serde(rename = "bittorrent")]
    BitTorrent,
    #[sea_orm(string_value = "poster")]
    #[serde(rename = "poster")]
    Poster,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "resources")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub origin_title: String,
    pub display_name: String,
    pub subscription_id: i32,
    pub status: DownloadStatus,
    pub category: ResourceCategory,
    pub url: String,
    pub all_size: Option<i64>,
    pub curr_size: Option<i64>,
    pub homepage: Option<String>,
    pub save_path: Option<String>,
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

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_mikan_rss_item(rss_item: MikanRssItem, subscription_id: i32) -> Self {
        let resource_category = rss_item.get_resource_category();
        Self {
            origin_title: ActiveValue::Set(rss_item.title.clone()),
            display_name: ActiveValue::Set(rss_item.title),
            subscription_id: ActiveValue::Set(subscription_id),
            status: ActiveValue::Set(DownloadStatus::Pending),
            category: ActiveValue::Set(resource_category),
            url: ActiveValue::Set(rss_item.url),
            all_size: ActiveValue::Set(rss_item.content_length),
            curr_size: ActiveValue::Set(Some(0)),
            homepage: ActiveValue::Set(rss_item.homepage),
            ..Default::default()
        }
    }

    pub fn from_poster_url(
        subscription_id: i32,
        origin_title: String,
        url: Url,
        save_path: Option<String>,
        content_length: i64,
    ) -> Self {
        Self {
            origin_title: ActiveValue::Set(origin_title.clone()),
            display_name: ActiveValue::Set(origin_title),
            subscription_id: ActiveValue::Set(subscription_id),
            status: ActiveValue::Set(DownloadStatus::Completed),
            category: ActiveValue::Set(ResourceCategory::Poster),
            url: ActiveValue::Set(url.to_string()),
            all_size: ActiveValue::Set(Some(content_length)),
            curr_size: ActiveValue::Set(Some(content_length)),
            save_path: ActiveValue::Set(save_path),
            ..Default::default()
        }
    }
}

impl Model {
    pub async fn from_poster_url<F, R, E>(
        ctx: &AppContext,
        subscriber_pid: &str,
        subscription_id: i32,
        original_title: String,
        url: Url,
        fetch_fn: F,
    ) -> eyre::Result<Self>
    where
        F: FnOnce(Url) -> R,
        R: Future<Output = Result<Bytes, E>>,
        E: Into<eyre::Report>,
    {
        let db = &ctx.db;
        let found = Entity::find()
            .filter(
                Column::SubscriptionId
                    .eq(subscription_id)
                    .and(Column::Url.eq(url.as_str())),
            )
            .one(db)
            .await?;

        let resource = if let Some(found) = found {
            found
        } else {
            let bytes = fetch_fn(url.clone()).await.map_err(|e| e.into())?;
            let content_length = bytes.len() as i64;
            let dal = ctx.get_dal_unwrap().await;
            let extname = extract_extname_from_url(&url)
                .ok_or_else(|| ParseError::ParseExtnameError(url.to_string()))?;
            let stored_url = dal
                .store_blob(DalContentType::Poster, &extname, bytes, subscriber_pid)
                .await?;
            let saved_path = Some(stored_url.to_string());

            let new_resource = ActiveModel::from_poster_url(
                subscription_id,
                original_title,
                url,
                saved_path,
                content_length,
            );

            let new_resource = new_resource.save(db).await?;
            new_resource.try_into_model()?
        };

        Ok(resource)
    }
}
