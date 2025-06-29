use async_trait::async_trait;
use sea_orm::entity::prelude::*;

pub use crate::task::{
    SubscriberTask, SubscriberTaskType, SubscriberTaskTypeEnum, SubscriberTaskTypeVariant,
    SubscriberTaskTypeVariantIter, subscriber_task_schema,
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SubscriberTaskStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
    #[sea_orm(string_value = "Scheduled")]
    Scheduled,
    #[sea_orm(string_value = "Running")]
    Running,
    #[sea_orm(string_value = "Done")]
    Done,
    #[sea_orm(string_value = "Failed")]
    Failed,
    #[sea_orm(string_value = "Killed")]
    Killed,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriber_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub subscriber_id: i32,
    pub subscription_id: Option<i32>,
    pub job: SubscriberTask,
    pub task_type: SubscriberTaskType,
    pub status: SubscriberTaskStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub run_at: DateTimeUtc,
    pub last_error: Option<String>,
    pub lock_at: Option<DateTimeUtc>,
    pub lock_by: Option<String>,
    pub done_at: Option<DateTimeUtc>,
    pub priority: i32,
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
    #[sea_orm(
        belongs_to = "super::subscriptions::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscriptions::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Subscription,
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
