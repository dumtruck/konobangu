use std::sync::Arc;

use sea_orm::{ActiveValue, FromJsonQueryResult, JsonValue, TryIntoModel, prelude::*};
use serde::{Deserialize, Serialize};

pub use crate::task::{SubscriberTaskType, SubscriberTaskTypeEnum};
use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    task::{SubscriberTask, SubscriberTaskPayload},
};

#[derive(Debug, Clone, Serialize, Deserialize, FromJsonQueryResult, PartialEq, Eq)]
pub struct SubscriberTaskErrorSnapshot {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, DeriveEntityModel, PartialEq, Eq)]
#[sea_orm(table_name = "subscriber_tasks")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscriber_id: i32,
    pub task_type: SubscriberTaskType,
    pub request: JsonValue,
    pub yields: Vec<JsonValue>,
    pub result: Option<JsonValue>,
    pub error: Option<JsonValue>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id"
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

impl Model {
    pub async fn update_result<R>(
        ctx: Arc<dyn AppContextTrait>,
        task_id: i32,
        result: R,
    ) -> RecorderResult<()>
    where
        R: Serialize,
    {
        let db = ctx.db();

        let result_value = serde_json::to_value(result)?;

        Entity::update_many()
            .filter(Column::Id.eq(task_id))
            .set(ActiveModel {
                result: ActiveValue::Set(Some(result_value)),
                ..Default::default()
            })
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn update_error(
        ctx: Arc<dyn AppContextTrait>,
        task_id: i32,
        error: SubscriberTaskErrorSnapshot,
    ) -> RecorderResult<()> {
        let db = ctx.db();

        let error_value = serde_json::to_value(&error)?;

        Entity::update_many()
            .filter(Column::Id.eq(task_id))
            .set(ActiveModel {
                error: ActiveValue::Set(Some(error_value)),
                ..Default::default()
            })
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn append_yield<Y>(
        ctx: Arc<dyn AppContextTrait>,
        task_id: i32,
        item: Y,
    ) -> RecorderResult<()>
    where
        Y: Serialize,
    {
        let db = ctx.db();

        let yield_value = serde_json::to_value(item)?;

        Entity::update_many()
            .filter(Column::Id.eq(task_id))
            .col_expr(
                Column::Yields,
                Expr::cust_with_values("array_append($1)", [yield_value]),
            )
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn add_subscriber_task(
        ctx: Arc<dyn AppContextTrait>,
        subscriber_id: i32,
        payload: SubscriberTaskPayload,
    ) -> RecorderResult<SubscriberTask> {
        let task_type = payload.task_type();
        let request: JsonValue = payload.clone().try_into()?;

        let am = ActiveModel {
            subscriber_id: ActiveValue::Set(subscriber_id),
            task_type: ActiveValue::Set(task_type.clone()),
            request: ActiveValue::Set(request.clone()),
            ..Default::default()
        };

        let db = ctx.db();

        let task_id = Entity::insert(am).exec(db).await?.last_insert_id;

        let subscriber_task = SubscriberTask {
            id: task_id,
            subscriber_id,
            payload,
        };

        ctx.task()
            .add_subscriber_task(subscriber_task.clone())
            .await?;

        Ok(subscriber_task)
    }
}
