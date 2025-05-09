use sea_orm_migration::{
    prelude::*,
    schema::{array, enumeration, integer, json_binary, json_binary_null, pk_auto},
};

use super::defs::{SubscriberTasks, Subscribers, table_auto_z};
use crate::models::subscriber_tasks::{SubscriberTaskType, SubscriberTaskTypeEnum};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_z(SubscriberTasks::Table)
                    .col(pk_auto(SubscriberTasks::Id))
                    .col(integer(SubscriberTasks::SubscriberId))
                    .col(enumeration(
                        SubscriberTasks::TaskType,
                        SubscriberTaskTypeEnum,
                        SubscriberTaskType::iden_values(),
                    ))
                    .col(json_binary(SubscriberTasks::Request))
                    .col(json_binary_null(SubscriberTasks::Result))
                    .col(json_binary_null(SubscriberTasks::Error))
                    .col(
                        array(SubscriberTasks::Yields, ColumnType::JsonBinary)
                            .default(SimpleExpr::Custom(String::from("ARRAY[]::jsonb[]"))),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriber_tasks_subscriber_id")
                            .from_tbl(SubscriberTasks::Table)
                            .from_col(SubscriberTasks::SubscriberId)
                            .to_tbl(Subscribers::Table)
                            .to_col(Subscribers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_subscriber_tasks_task_type")
                    .table(SubscriberTasks::Table)
                    .col(SubscriberTasks::TaskType)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_subscriber_tasks_task_type")
                    .table(SubscriberTasks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(SubscriberTasks::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
