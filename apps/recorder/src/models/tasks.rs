use async_trait::async_trait;
use sea_orm::{QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::{app::AppContextTrait, errors::app_error::RResult};

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[sea_orm(string_value = "p")]
    Pending,
    #[sea_orm(string_value = "r")]
    Running,
    #[sea_orm(string_value = "s")]
    Success,
    #[sea_orm(string_value = "f")]
    Failed,
}

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
#[serde(rename_all = "snake_case")]
pub enum TaskMode {
    #[sea_orm(string_value = "stream")]
    Stream,
    #[sea_orm(string_value = "future")]
    Future,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscriber_id: i32,
    pub task_mode: TaskMode,
    pub task_status: TaskStatus,
    pub task_type: String,
    pub state_data: serde_json::Value,
    pub request_data: serde_json::Value,
    pub error_data: serde_json::Value,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task_stream_item::Entity")]
    StreamItem,
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

impl Related<super::task_stream_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StreamItem.def()
    }
}

impl Model {
    pub async fn find_stream_task_by_id(
        ctx: &dyn AppContextTrait,
        task_id: i32,
    ) -> RResult<Option<(Model, Vec<super::task_stream_item::Model>)>> {
        let db = ctx.db();
        let res = Entity::find()
            .filter(Column::Id.eq(task_id))
            .filter(Column::TaskMode.eq(TaskMode::Stream))
            .find_with_related(super::task_stream_item::Entity)
            .limit(1)
            .all(db)
            .await?
            .pop();

        Ok(res)
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
