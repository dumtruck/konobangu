use async_trait::async_trait;
use sea_orm::{ActiveValue, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::{app::AppContextTrait, errors::RecorderResult};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscription_episode")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscriber_id: i32,
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

impl Model {
    pub async fn add_episodes_for_subscription(
        ctx: &dyn AppContextTrait,
        episode_ids: impl Iterator<Item = i32>,
        subscriber_id: i32,
        subscription_id: i32,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        Entity::insert_many(episode_ids.map(|episode_id| ActiveModel {
            episode_id: ActiveValue::Set(episode_id),
            subscription_id: ActiveValue::Set(subscription_id),
            subscriber_id: ActiveValue::Set(subscriber_id),
            ..Default::default()
        }))
        .on_conflict_do_nothing()
        .exec(db)
        .await?;

        Ok(())
    }
}
