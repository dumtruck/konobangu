use async_trait::async_trait;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscription_episode")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscription_id: i32,
    pub episode_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscriptions::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscriptions::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Subscription,
    #[sea_orm(
        belongs_to = "super::episodes::Entity",
        from = "Column::EpisodeId",
        to = "super::episodes::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(entity = "super::episodes::Entity")]
    Episode,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_subscription_and_episode(subscription_id: i32, episode_id: i32) -> Self {
        Self {
            subscription_id: ActiveValue::Set(subscription_id),
            episode_id: ActiveValue::Set(episode_id),
            ..Default::default()
        }
    }
}
