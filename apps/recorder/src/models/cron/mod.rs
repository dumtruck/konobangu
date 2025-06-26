mod core;
mod registry;

pub use core::{
    CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME, CRON_DUE_EVENT,
    NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME, NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use croner::Cron;
use sea_orm::{
    ActiveValue::Set, DeriveActiveEnum, DeriveDisplay, DeriveEntityModel, EnumIter, QuerySelect,
    Statement, TransactionTrait, entity::prelude::*, sea_query::LockType,
    sqlx::postgres::PgNotification,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    models::subscriptions::{self},
};

#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cron_source")]
#[serde(rename_all = "snake_case")]
pub enum CronSource {
    #[sea_orm(string_value = "subscription")]
    Subscription,
}

#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cron_status")]
#[serde(rename_all = "snake_case")]
pub enum CronStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cron")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub cron_source: CronSource,
    pub subscriber_id: Option<i32>,
    pub subscription_id: Option<i32>,
    pub cron_expr: String,
    pub next_run: Option<DateTimeUtc>,
    pub last_run: Option<DateTimeUtc>,
    pub last_error: Option<String>,
    pub locked_by: Option<String>,
    pub locked_at: Option<DateTimeUtc>,
    pub timeout_ms: i32,
    #[sea_orm(default_expr = "0")]
    pub attempts: i32,
    #[sea_orm(default_expr = "1")]
    pub max_attempts: i32,
    #[sea_orm(default_expr = "0")]
    pub priority: i32,
    pub status: CronStatus,
    #[sea_orm(default_expr = "true")]
    pub enabled: bool,
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
        on_update = "Cascade",
        on_delete = "Cascade"
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

impl Model {
    pub async fn handle_cron_notification(
        ctx: &dyn AppContextTrait,
        notification: PgNotification,
        worker_id: &str,
    ) -> RecorderResult<()> {
        let payload: Self = serde_json::from_str(notification.payload())?;
        let cron_id = payload.id;

        tracing::debug!("Cron notification received for cron {cron_id} and worker {worker_id}");

        match Self::try_acquire_lock_with_cron_id(ctx, cron_id, worker_id).await? {
            Some(cron) => match cron.exec_cron(ctx).await {
                Ok(()) => {
                    tracing::debug!("Cron {cron_id} executed successfully");
                    cron.mark_cron_completed(ctx).await?;
                }
                Err(e) => {
                    tracing::error!("Error executing cron {cron_id}: {e}");
                    cron.mark_cron_failed(ctx, &e.to_string()).await?;
                }
            },
            None => {
                tracing::debug!(
                    "Cron lock not acquired for cron {cron_id} and worker {worker_id}, skipping..."
                );
            }
        }

        Ok(())
    }

    async fn try_acquire_lock_with_cron_id(
        ctx: &dyn AppContextTrait,
        cron_id: i32,
        worker_id: &str,
    ) -> RecorderResult<Option<Self>> {
        let db = ctx.db();
        let txn = db.begin().await?;

        let cron = Entity::find_by_id(cron_id)
            .lock(LockType::Update)
            .one(&txn)
            .await?;

        if let Some(cron) = cron {
            if cron.enabled
                && cron.attempts < cron.max_attempts
                && cron.status == CronStatus::Pending
                && (cron.locked_at.is_none_or(|locked_at| {
                    locked_at + chrono::Duration::milliseconds(cron.timeout_ms as i64) <= Utc::now()
                }))
                && cron.next_run.is_some_and(|next_run| next_run <= Utc::now())
            {
                let cron_active_model = ActiveModel {
                    id: Set(cron.id),
                    locked_by: Set(Some(worker_id.to_string())),
                    locked_at: Set(Some(Utc::now())),
                    status: Set(CronStatus::Running),
                    attempts: Set(cron.attempts + 1),
                    ..Default::default()
                };
                let cron_model = cron_active_model.update(&txn).await?;
                txn.commit().await?;
                return Ok(Some(cron_model));
            }
            txn.commit().await?;
            return Ok(Some(cron));
        }
        txn.rollback().await?;
        Ok(None)
    }

    async fn exec_cron(&self, ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        match self.cron_source {
            CronSource::Subscription => {
                let subscription_id = self.subscription_id.unwrap_or_else(|| {
                    unreachable!("Subscription cron must have a subscription id")
                });

                let subscription = subscriptions::Entity::find_by_id(subscription_id)
                    .one(ctx.db())
                    .await?
                    .ok_or_else(|| RecorderError::from_model_not_found("Subscription"))?;

                subscription.exec_cron(ctx).await?;
            }
        }

        Ok(())
    }

    async fn mark_cron_completed(&self, ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        let db = ctx.db();

        let next_run = self.calculate_next_run(&self.cron_expr)?;

        ActiveModel {
            id: Set(self.id),
            next_run: Set(Some(next_run)),
            last_run: Set(Some(Utc::now())),
            status: Set(CronStatus::Pending),
            locked_by: Set(None),
            locked_at: Set(None),
            attempts: Set(0),
            last_error: Set(None),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(())
    }

    async fn mark_cron_failed(&self, ctx: &dyn AppContextTrait, error: &str) -> RecorderResult<()> {
        let db = ctx.db();

        let should_retry = self.attempts < self.max_attempts;

        let status = if should_retry {
            CronStatus::Pending
        } else {
            CronStatus::Failed
        };

        let next_run = if should_retry {
            Some(Utc::now() + chrono::Duration::seconds(5))
        } else {
            Some(self.calculate_next_run(&self.cron_expr)?)
        };

        ActiveModel {
            id: Set(self.id),
            next_run: Set(next_run),
            status: Set(status),
            locked_by: Set(None),
            locked_at: Set(None),
            last_run: Set(Some(Utc::now())),
            last_error: Set(Some(error.to_string())),
            attempts: Set(if should_retry { self.attempts + 1 } else { 0 }),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(())
    }

    pub async fn check_and_trigger_due_crons(ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        let db = ctx.db();

        db.execute(Statement::from_string(
            db.get_database_backend(),
            format!("SELECT {CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME}()"),
        ))
        .await?;

        Ok(())
    }

    fn calculate_next_run(&self, cron_expr: &str) -> RecorderResult<DateTime<Utc>> {
        let cron_expr = Cron::new(cron_expr).parse()?;

        let next = cron_expr.find_next_occurrence(&Utc::now(), false)?;

        Ok(next)
    }
}
