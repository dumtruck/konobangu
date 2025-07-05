use async_trait::async_trait;
use sea_orm::{ActiveValue, entity::prelude::*};

use crate::task::SubscriberTaskTrait;
pub use crate::task::{
    SubscriberTask, SubscriberTaskInput, SubscriberTaskType, SubscriberTaskTypeEnum,
    SubscriberTaskTypeVariant, SubscriberTaskTypeVariantIter,
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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriber_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub subscriber_id: i32,
    pub subscription_id: Option<i32>,
    pub cron_id: Option<i32>,
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
        on_delete = "Restrict"
    )]
    Subscriber,
    #[sea_orm(
        belongs_to = "super::subscriptions::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscriptions::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Subscription,
    #[sea_orm(
        belongs_to = "super::cron::Entity",
        from = "Column::CronId",
        to = "super::cron::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Cron,
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

impl Related<super::cron::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cron.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(entity = "super::cron::Entity")]
    Cron,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if let ActiveValue::Set(subscriber_id) = self.subscriber_id
            && let ActiveValue::Set(ref job) = self.job
            && job.get_subscriber_id() != subscriber_id
        {
            return Err(DbErr::Custom(
                "SubscriberTask subscriber_id does not match job.subscriber_id".to_string(),
            ));
        }
        Ok(self)
    }
}
