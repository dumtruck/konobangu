use sea_orm_migration::prelude::*;

use super::defs::{Cron, Subscribers, Subscriptions, TaskOutbox, TaskRuns, TaskViews};
use crate::database::roles::TASK_CONTROL_ACCESS_ROLE;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(&schema_sql()).await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration(
      "Task delivery is not reversible after new writes; stop services and restore a backup".into(),
    ))
  }
}

// PostgreSQL policies and partial indexes need native SQL.
// Names come from the same Rust identifiers as the typed migration queries.
fn schema_sql() -> String {
  format!(
    r#"
CREATE TABLE {task_runs} (
  {id} text PRIMARY KEY,
  {kind} text NOT NULL CHECK ({kind} IN ('subscriber_task', 'system_task')),
  {subscriber_id} integer REFERENCES {subscribers}({subscribers_id}) ON DELETE RESTRICT,
  {subscription_id} integer REFERENCES {subscriptions}({subscriptions_id}) ON DELETE SET NULL,
  {cron_id} integer REFERENCES {cron}({cron_target_id}) ON DELETE SET NULL,
  {scheduled_at_utc} timestamptz,
  {payload_version} integer NOT NULL DEFAULT 1 CHECK ({payload_version} = 1),
  {payload} jsonb NOT NULL,
  {task_type} text NOT NULL,
  {status} text NOT NULL DEFAULT 'Pending' CHECK ({status} IN ('Pending','Scheduled','Running','Done','Failed','Killed')),
  {generation} bigint NOT NULL DEFAULT 1 CHECK ({generation} > 0),
  {attempts} integer NOT NULL DEFAULT 0 CHECK ({attempts} >= 0),
  {max_attempts} integer NOT NULL DEFAULT 3 CHECK ({max_attempts} > 0),
  {run_at} timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  {done_at} timestamptz,
  {last_error} text,
  {cancel_requested_at} timestamptz,
  {archived_at} timestamptz,
  {execution_token} uuid,
  {execution_lease_until} timestamptz,
  {created_at} timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK ({kind} <> 'subscriber_task' OR {subscriber_id} IS NOT NULL),
  UNIQUE ({cron_id}, {scheduled_at_utc})
);
CREATE INDEX {task_runs}_owner ON {task_runs}({subscriber_id}, {run_at} DESC) WHERE {archived_at} IS NULL;
CREATE INDEX {task_runs}_recovery ON {task_runs}({execution_lease_until}) WHERE {status} = 'Running';
CREATE TABLE {task_outbox} (
  {task_id} text NOT NULL REFERENCES {task_runs}({id}) ON DELETE RESTRICT,
  {generation} bigint NOT NULL,
  {available_at} timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  {lease_token} uuid,
  {lease_until} timestamptz,
  {attempts} integer NOT NULL DEFAULT 0,
  {last_error} text,
  {sent_at} timestamptz,
  PRIMARY KEY ({task_id}, {generation})
);
CREATE INDEX {task_outbox}_due ON {task_outbox}({available_at}) WHERE {sent_at} IS NULL;

ALTER TABLE {task_runs} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {task_runs} FORCE ROW LEVEL SECURITY;
CREATE POLICY {task_runs}_app_scoped ON {task_runs} USING (
  {subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer
) WITH CHECK ({subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer);
CREATE POLICY {task_runs}_task_control ON {task_runs} TO {task_control_access} USING (true) WITH CHECK (true);
ALTER TABLE {task_outbox} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {task_outbox} FORCE ROW LEVEL SECURITY;
CREATE POLICY {task_outbox}_app_scoped ON {task_outbox} USING (
  EXISTS (SELECT 1 FROM {task_runs} t WHERE t.{id} = {task_id} AND t.{subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer)
) WITH CHECK (
  EXISTS (SELECT 1 FROM {task_runs} t WHERE t.{id} = {task_id} AND t.{subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer)
);
CREATE POLICY {task_outbox}_task_control ON {task_outbox} TO {task_control_access} USING (true) WITH CHECK (true);
CREATE VIEW {subscriber_tasks} WITH (security_invoker = true) AS
  SELECT {id}, {subscriber_id}, {subscription_id}, {cron_id}, {payload} AS {job}, {task_type},
    {status}, {generation}, {attempts}, {max_attempts}, {run_at}, {last_error}, {done_at}, {cancel_requested_at}
  FROM {task_runs} WHERE {kind} = 'subscriber_task' AND {archived_at} IS NULL;
CREATE VIEW {system_tasks} WITH (security_invoker = true) AS
  SELECT {id}, {subscriber_id}, {cron_id}, {payload} AS {job}, {task_type},
    {status}, {generation}, {attempts}, {max_attempts}, {run_at}, {last_error}, {done_at}, {cancel_requested_at}
  FROM {task_runs} WHERE {kind} = 'system_task' AND {archived_at} IS NULL;
"#,
    archived_at = TaskRuns::ArchivedAt.to_string(),
    attempts = TaskRuns::Attempts.to_string(),
    available_at = TaskOutbox::AvailableAt.to_string(),
    cancel_requested_at = TaskRuns::CancelRequestedAt.to_string(),
    created_at = TaskRuns::CreatedAt.to_string(),
    cron = Cron::Table.to_string(),
    cron_id = TaskRuns::CronId.to_string(),
    cron_target_id = Cron::Id.to_string(),
    done_at = TaskRuns::DoneAt.to_string(),
    execution_lease_until = TaskRuns::ExecutionLeaseUntil.to_string(),
    execution_token = TaskRuns::ExecutionToken.to_string(),
    generation = TaskRuns::Generation.to_string(),
    id = TaskRuns::Id.to_string(),
    job = TaskViews::Job.to_string(),
    kind = TaskRuns::Kind.to_string(),
    last_error = TaskRuns::LastError.to_string(),
    lease_token = TaskOutbox::LeaseToken.to_string(),
    lease_until = TaskOutbox::LeaseUntil.to_string(),
    max_attempts = TaskRuns::MaxAttempts.to_string(),
    payload = TaskRuns::Payload.to_string(),
    payload_version = TaskRuns::PayloadVersion.to_string(),
    run_at = TaskRuns::RunAt.to_string(),
    scheduled_at_utc = TaskRuns::ScheduledAtUtc.to_string(),
    sent_at = TaskOutbox::SentAt.to_string(),
    status = TaskRuns::Status.to_string(),
    subscriber_id = TaskRuns::SubscriberId.to_string(),
    subscriber_tasks = TaskViews::SubscriberTasks.to_string(),
    subscribers = Subscribers::Table.to_string(),
    subscribers_id = Subscribers::Id.to_string(),
    subscription_id = TaskRuns::SubscriptionId.to_string(),
    subscriptions = Subscriptions::Table.to_string(),
    subscriptions_id = Subscriptions::Id.to_string(),
    system_tasks = TaskViews::SystemTasks.to_string(),
    task_id = TaskOutbox::TaskId.to_string(),
    task_outbox = TaskOutbox::Table.to_string(),
    task_runs = TaskRuns::Table.to_string(),
    task_type = TaskRuns::TaskType.to_string(),
    task_control_access = TASK_CONTROL_ACCESS_ROLE,
  )
}
