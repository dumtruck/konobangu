use sea_orm::entity::prelude::*;

use crate::task::SubscriberTask;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriber_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub subscriber_id: i32,
    pub job: SubscriberTask,
    pub status: String,
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
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
}

impl ActiveModelBehavior for ActiveModel {}
