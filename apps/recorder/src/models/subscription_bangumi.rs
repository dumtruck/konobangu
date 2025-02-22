use async_trait::async_trait;
use sea_orm::{ActiveValue, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscription_bangumi")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscriber_id: i32,
    pub subscription_id: i32,
    pub bangumi_id: i32,
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
        belongs_to = "super::bangumi::Entity",
        from = "Column::BangumiId",
        to = "super::bangumi::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Bangumi,
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_subscription_and_bangumi(
        subscriber_id: i32,
        subscription_id: i32,
        bangumi_id: i32,
    ) -> Self {
        Self {
            subscriber_id: ActiveValue::Set(subscriber_id),
            subscription_id: ActiveValue::Set(subscription_id),
            bangumi_id: ActiveValue::Set(bangumi_id),
            ..Default::default()
        }
    }
}
