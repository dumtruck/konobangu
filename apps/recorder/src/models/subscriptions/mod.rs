mod core;
mod registry;
pub use core::SubscriptionTrait;
use std::fmt::Debug;

use async_trait::async_trait;
pub use registry::{
    Subscription, SubscriptionCategory, SubscriptionCategoryEnum, SubscriptionCategoryIter,
    SubscriptionCategoryVariant, SubscriptionCategoryVariantIter,
};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    pub subscriber_id: i32,
    pub category: SubscriptionCategory,
    pub source_url: String,
    pub enabled: bool,
    pub credential_id: Option<i32>,
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
    #[sea_orm(has_many = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episodes,
    #[sea_orm(has_many = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
    #[sea_orm(has_many = "super::subscription_bangumi::Entity")]
    SubscriptionBangumi,
    #[sea_orm(
        belongs_to = "super::credential_3rd::Entity",
        from = "Column::CredentialId",
        to = "super::credential_3rd::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Credential3rd,
    #[sea_orm(has_many = "super::feeds::Entity")]
    Feed,
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::subscription_bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionBangumi.def()
    }
}

impl Related<super::subscription_episode::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionEpisode.def()
    }
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_bangumi::Relation::Bangumi.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::subscription_bangumi::Relation::Subscription
                .def()
                .rev(),
        )
    }
}

impl Related<super::feeds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feed.def()
    }
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_episode::Relation::Episode.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::subscription_episode::Relation::Subscription
                .def()
                .rev(),
        )
    }
}

impl Related<super::credential_3rd::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Credential3rd.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(entity = "super::episodes::Entity")]
    Episode,
    #[sea_orm(entity = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
    #[sea_orm(entity = "super::subscription_bangumi::Entity")]
    SubscriptionBangumi,
    #[sea_orm(entity = "super::credential_3rd::Entity")]
    Credential3rd,
    #[sea_orm(entity = "super::feeds::Entity")]
    Feed,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {}

impl Model {
    pub async fn toggle_with_ids(
        ctx: &dyn AppContextTrait,
        ids: impl Iterator<Item = i32>,
        enabled: bool,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        Entity::update_many()
            .col_expr(Column::Enabled, Expr::value(enabled))
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_with_ids(
        ctx: &dyn AppContextTrait,
        ids: impl Iterator<Item = i32>,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn find_by_id_and_subscriber_id(
        ctx: &dyn AppContextTrait,
        subscriber_id: i32,
        subscription_id: i32,
    ) -> RecorderResult<Self> {
        let db = ctx.db();
        let subscription_model = Entity::find_by_id(subscription_id)
            .one(db)
            .await?
            .ok_or_else(|| RecorderError::ModelEntityNotFound {
                entity: "Subscription".into(),
            })?;

        if subscription_model.subscriber_id != subscriber_id {
            Err(RecorderError::ModelEntityNotFound {
                entity: "Subscription".into(),
            })?;
        }

        Ok(subscription_model)
    }
}
