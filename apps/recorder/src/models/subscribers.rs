use async_graphql::SimpleObject;
use async_trait::async_trait;
use sea_orm::{ActiveValue, FromJsonQueryResult, TransactionTrait, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::app_error::{RecorderResult, RecorderError},
};

pub const SEED_SUBSCRIBER: &str = "konobangu";

#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, SimpleObject,
)]
pub struct SubscriberBangumiConfig {
    pub leading_group_tag: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "subscribers")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    pub bangumi_conf: Option<SubscriberBangumiConfig>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(has_many = "super::downloaders::Entity")]
    Downloader,
    #[sea_orm(has_many = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episode,
    #[sea_orm(has_many = "super::auth::Entity")]
    Auth,
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::downloaders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Downloader.def()
    }
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Episode.def()
    }
}

impl Related<super::auth::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Auth.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(entity = "super::downloaders::Entity")]
    Downloader,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(entity = "super::episodes::Entity")]
    Episode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscriberIdParams {
    pub id: String,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn find_seed_subscriber_id(ctx: &dyn AppContextTrait) -> RecorderResult<i32> {
        let subscriber_auth = crate::models::auth::Model::find_by_pid(ctx, SEED_SUBSCRIBER).await?;
        Ok(subscriber_auth.subscriber_id)
    }

    pub async fn find_by_id(ctx: &dyn AppContextTrait, id: i32) -> RecorderResult<Self> {
        let db = ctx.db();

        let subscriber = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| RecorderError::from_db_record_not_found("subscriptions::find_by_id"))?;
        Ok(subscriber)
    }

    pub async fn create_root(ctx: &dyn AppContextTrait) -> RecorderResult<Self> {
        let db = ctx.db();
        let txn = db.begin().await?;

        let user = ActiveModel {
            display_name: ActiveValue::set(SEED_SUBSCRIBER.to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }
}
