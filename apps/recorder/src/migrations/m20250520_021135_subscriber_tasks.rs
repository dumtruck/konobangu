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
            r#"CREATE OR REPLACE VIEW subscriber_task AS
SELECT
    job,
    job_type,
    status,
    (job->'subscriber_id')::integer AS subscriber_id,
    (job->'task_type')::text AS task_type,
    id,
    attempts,
    max_attempts,
    run_at,
    last_error,
    lock_at,
    lock_by,
    done_at,
    priority
FROM apalis.jobs
WHERE job_type = '{SUBSCRIBER_TASK_APALIS_NAME}'
AND jsonb_path_exists(job, '$.subscriber_id ? (@.type() == "number")')
AND jsonb_path_exists(job, '$.task_type ? (@.type() == "string")')"#,
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"CREATE INDEX IF NOT EXISTS idx_apalis_jobs_subscriber_id
             ON apalis.jobs ((job -> 'subscriber_id'))
             WHERE job_type = '{SUBSCRIBER_TASK_APALIS_NAME}'
AND jsonb_path_exists(job, '$.subscriber_id ? (@.type() == "number")')
AND jsonb_path_exists(job, '$.task_type ? (@.type() == "string")')"#
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"DROP INDEX IF EXISTS idx_apalis_jobs_subscriber_id
             ON apalis.jobs"#,
        )
        .await?;

        db.execute_unprepared("DROP VIEW IF EXISTS subscriber_task")
            .await?;

        Ok(())
    }
}
