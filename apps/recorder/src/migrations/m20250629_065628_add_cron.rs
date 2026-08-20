use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
  migrations::defs::{Cron, CustomSchemaManagerExt, GeneralIds, Subscribers, Subscriptions, table_auto_z},
  models::cron::{SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        table_auto_z(Cron::Table)
          .col(pk_auto(Cron::Id))
          .col(string(Cron::CronExpr))
          .col(string(Cron::CronTimezone))
          .col(integer_null(Cron::SubscriberId))
          .col(integer_null(Cron::SubscriptionId))
          .col(timestamp_with_time_zone_null(Cron::NextRun))
          .col(timestamp_with_time_zone_null(Cron::LastRun))
          .col(string_null(Cron::LastError))
          .col(boolean(Cron::Enabled).default(true))
          .col(json_binary_null(Cron::SubscriberTaskCron))
          .col(json_binary_null(Cron::SystemTaskCron))
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
            DECLARE
                new_subscriber_task_subscriber_id integer;
                new_subscriber_task_subscription_id integer;
                new_system_task_subscriber_id integer;
            BEGIN
                new_subscriber_task_subscriber_id = (NEW.{subscriber_task_cron} ->> 'subscriber_id')::integer;
                new_subscriber_task_subscription_id = (NEW.{subscriber_task_cron} ->> 'subscription_id')::integer;
                new_system_task_subscriber_id = (NEW.{system_task_cron} ->> 'subscriber_id')::integer;
                IF new_subscriber_task_subscriber_id IS DISTINCT FROM (OLD.{subscriber_task_cron} ->> 'subscriber_id')::integer AND new_subscriber_task_subscriber_id IS DISTINCT FROM NEW.{subscriber_id} THEN
                    NEW.{subscriber_id} = new_subscriber_task_subscriber_id;
                END IF;
                IF new_subscriber_task_subscription_id IS DISTINCT FROM (OLD.{subscriber_task_cron} ->> 'subscription_id')::integer AND new_subscriber_task_subscription_id IS DISTINCT FROM NEW.{subscription_id} THEN
                    NEW.{subscription_id} = new_subscriber_task_subscription_id;
                END IF;
                IF new_system_task_subscriber_id IS DISTINCT FROM (OLD.{system_task_cron} ->> 'subscriber_id')::integer AND new_system_task_subscriber_id IS DISTINCT FROM NEW.{subscriber_id} THEN
                    NEW.{subscriber_id} = new_system_task_subscriber_id;
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;"#,
            subscriber_task_cron = Cron::SubscriberTaskCron.to_string(),
            subscriber_id = Cron::SubscriberId.to_string(),
            subscription_id = Cron::SubscriptionId.to_string(),
            system_task_cron = Cron::SystemTaskCron.to_string(),
        )).await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE TRIGGER {SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME}
                BEFORE INSERT OR UPDATE ON {table}
                FOR EACH ROW
                EXECUTE FUNCTION {SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}();"#,
      table = Cron::Table.to_string(),
    ))
    .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(TableDropStatement::new().if_exists().table(Cron::Table).to_owned()).await?;
    manager
      .get_connection()
      .execute_unprepared(&format!("DROP FUNCTION IF EXISTS {SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}()"))
      .await?;
    Ok(())
  }
}
