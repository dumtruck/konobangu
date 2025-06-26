use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::task::SUBSCRIBER_TASK_APALIS_NAME;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE VIEW subscriber_tasks AS
SELECT
    job,
    job_type,
    status,
    (job ->> 'subscriber_id')::integer AS subscriber_id,
    job ->> 'task_type'                AS task_type,
    id,
    attempts,
    max_attempts,
    run_at,
    last_error,
    lock_at,
    lock_by,
    done_at,
    priority,
    (job ->> 'subscription_id')::integer AS subscription_id
FROM apalis.jobs
WHERE job_type = '{SUBSCRIBER_TASK_APALIS_NAME}'
AND jsonb_path_exists(job, '$.subscriber_id ? (@.type() == "number")')
AND jsonb_path_exists(job, '$.task_type ? (@.type() == "string")')"#,
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"CREATE INDEX IF NOT EXISTS idx_apalis_jobs_subscription_id
                     ON apalis.jobs (((job -> 'subscription_id')::integer))
                     WHERE job_type = '{SUBSCRIBER_TASK_APALIS_NAME}'
        AND jsonb_path_exists(job, '$.subscription_id ? (@.type() == "number")')
        AND jsonb_path_exists(job, '$.task_type ? (@.type() == "string")')"#
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"DROP INDEX IF EXISTS idx_apalis_jobs_subscription_id
             ON apalis.jobs"#,
        )
        .await?;

        Ok(())
    }
}
