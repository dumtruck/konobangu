use async_trait::async_trait;
use sea_orm::ActiveEnum;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    migrations::defs::{
        Cron, CustomSchemaManagerExt, GeneralIds, Subscribers, Subscriptions, table_auto_z,
    },
    models::cron::{
        CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME, CRON_DUE_EVENT, CronStatus, CronStatusEnum,
        NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME, NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME,
        SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_postgres_enum_for_active_enum!(
            manager,
            CronStatusEnum,
            CronStatus::Pending,
            CronStatus::Running,
            CronStatus::Completed,
            CronStatus::Failed
        )
        .await?;

        manager
            .create_table(
                table_auto_z(Cron::Table)
                    .col(pk_auto(Cron::Id))
                    .col(string(Cron::CronExpr))
                    .col(integer_null(Cron::SubscriberId))
                    .col(integer_null(Cron::SubscriptionId))
                    .col(timestamp_with_time_zone_null(Cron::NextRun))
                    .col(timestamp_with_time_zone_null(Cron::LastRun))
                    .col(string_null(Cron::LastError))
                    .col(boolean(Cron::Enabled).default(true))
                    .col(string_null(Cron::LockedBy))
                    .col(timestamp_with_time_zone_null(Cron::LockedAt))
                    .col(integer_null(Cron::TimeoutMs))
                    .col(integer(Cron::Attempts))
                    .col(integer(Cron::MaxAttempts))
                    .col(integer(Cron::Priority))
                    .col(enumeration(
                        Cron::Status,
                        CronStatusEnum,
                        CronStatus::iden_values(),
                    ))
                    .col(json_binary_null(Cron::SubscriberTask))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cron_subscriber_id")
                            .from(Cron::Table, Cron::SubscriberId)
                            .to(Subscribers::Table, Subscribers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cron_subscription_id")
                            .from(Cron::Table, Cron::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(Cron::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .if_not_exists()
                    .name("idx_cron_next_run")
                    .table(Cron::Table)
                    .col(Cron::NextRun)
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}() RETURNS trigger AS $$
            BEGIN
                IF jsonb_path_exists(NEW.{subscriber_task}, '$.subscriber_id ? (@.type() == "number")') THEN
                    NEW.{subscriber_id} = (NEW.{subscriber_task} ->> 'subscriber_id')::integer;
                END IF;
                IF jsonb_path_exists(NEW.{subscriber_task}, '$.subscription_id ? (@.type() == "number")') THEN
                    NEW.{subscription_id} = (NEW.{subscriber_task} ->> 'subscription_id')::integer;
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;"#,
            subscriber_task = &Cron::SubscriberTask.to_string(),
            subscriber_id = &Cron::SubscriberId.to_string(),
            subscription_id = &Cron::SubscriptionId.to_string(),
        )).await?;

        db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE TRIGGER {SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME}
                BEFORE INSERT OR UPDATE ON {table}
                FOR EACH ROW
                EXECUTE FUNCTION {SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}();"#,
            table = &Cron::Table.to_string(),
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME}() RETURNS trigger AS $$
        BEGIN
            -- Check if the cron is due to run
            IF NEW.{next_run} IS NOT NULL
               AND NEW.{next_run} <= CURRENT_TIMESTAMP
               AND NEW.{enabled} = true
               AND NEW.{status} = '{pending}'
               AND NEW.{attempts} < NEW.{max_attempts}
               -- Check if not locked or lock timeout
               AND (
                  NEW.{locked_at} IS NULL
                  OR (
                    NEW.{timeout_ms} IS NOT NULL
                    AND (NEW.{locked_at} + NEW.{timeout_ms} * INTERVAL '1 millisecond') <= CURRENT_TIMESTAMP
                  )
               )
               -- Make sure the cron is a new due event, not a repeat event
               AND (
                  OLD.{next_run} IS NULL
                  OR OLD.{next_run} > CURRENT_TIMESTAMP
                  OR OLD.{enabled} = false
                  OR OLD.{status} != '{pending}'
                  OR OLD.{attempts} != NEW.{attempts}
               )
               THEN
                  PERFORM pg_notify('{CRON_DUE_EVENT}', row_to_json(NEW)::text);
            END IF;
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;"#,
            next_run = &Cron::NextRun.to_string(),
            enabled = &Cron::Enabled.to_string(),
            locked_at = &Cron::LockedAt.to_string(),
            timeout_ms = &Cron::TimeoutMs.to_string(),
            status = &Cron::Status.to_string(),
            pending = &CronStatus::Pending.to_value(),
            attempts = &Cron::Attempts.to_string(),
            max_attempts = &Cron::MaxAttempts.to_string(),
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE TRIGGER {NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME}
                AFTER INSERT OR UPDATE ON {table}
                FOR EACH ROW
                EXECUTE FUNCTION {NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME}();"#,
            table = &Cron::Table.to_string(),
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME}() RETURNS INTEGER AS $$
            DECLARE
                cron_record RECORD;
                notification_count INTEGER := 0;
            BEGIN
                FOR cron_record IN
                    SELECT * FROM {table}
                    WHERE {next_run} IS NOT NULL
                        AND {next_run} <= CURRENT_TIMESTAMP
                        AND {enabled} = true
                        AND {status} = '{pending}'
                        AND {attempts} < {max_attempts}
                        AND (
                            {locked_at} IS NULL
                            OR (
                                {timeout_ms} IS NOT NULL
                                AND {locked_at} + {timeout_ms} * INTERVAL '1 millisecond' <= CURRENT_TIMESTAMP
                            )
                        )
                    ORDER BY {priority} ASC, {next_run} ASC
                    FOR UPDATE SKIP LOCKED
                LOOP
                    PERFORM pg_notify('{CRON_DUE_EVENT}', row_to_json(cron_record)::text);
                    notification_count := notification_count + 1;
                END LOOP;
                RETURN notification_count;
            END;
            $$ LANGUAGE plpgsql;"#,
            table = &Cron::Table.to_string(),
            next_run = &Cron::NextRun.to_string(),
            enabled = &Cron::Enabled.to_string(),
            status = &Cron::Status.to_string(),
            pending = &CronStatus::Pending.to_value(),
            locked_at = &Cron::LockedAt.to_string(),
            timeout_ms = &Cron::TimeoutMs.to_string(),
            priority = &Cron::Priority.to_string(),
            attempts = &Cron::Attempts.to_string(),
            max_attempts = &Cron::MaxAttempts.to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(&format!(
            r#"DROP TRIGGER IF EXISTS {NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME} ON {table};"#,
            table = &Cron::Table.to_string(),
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"DROP FUNCTION IF EXISTS {NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME}();"#,
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"DROP FUNCTION IF EXISTS {CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME}();"#,
        ))
        .await?;

        manager
            .drop_table(
                TableDropStatement::new()
                    .if_exists()
                    .table(Cron::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(CronStatusEnum)
            .await?;

        Ok(())
    }
}
