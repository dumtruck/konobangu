use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[sea_orm(string_value = "r")]
    Running,
    #[sea_orm(string_value = "s")]
    Success,
    #[sea_orm(string_value = "f")]
    Failed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub task_id: i32,
    pub subscriber_id: i32,
    pub item: serde_json::Value,
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
        belongs_to = "super::tasks::Entity",
        from = "Column::TaskId",
        to = "super::tasks::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
