use sea_orm::entity::prelude::*;

use crate::task::SubscriberTask;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriber_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscriber_id: i32,
    pub job: SubscriberTask,
    pub state: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
