use std::{future::Future, ops::Deref, str::FromStr, sync::Arc};

use apalis::prelude::*;
use apalis_sql::{
    Config,
    context::SqlContext,
    postgres::{PgListen as ApalisPgListen, PostgresStorage as ApalisPostgresStorage},
};
use sea_orm::{ActiveModelTrait, sqlx::postgres::PgListener};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    models::cron::{self, CRON_DUE_EVENT},
    task::{
        AsyncTaskTrait, SUBSCRIBER_TASK_APALIS_NAME, SYSTEM_TASK_APALIS_NAME, SubscriberTask,
        TaskConfig,
        config::{default_subscriber_task_workers, default_system_task_workers},
        registry::SystemTask,
    },
};

pub struct TaskService {
    pub config: TaskConfig,
    ctx: Arc<dyn AppContextTrait>,
    subscriber_task_storage: Arc<RwLock<ApalisPostgresStorage<SubscriberTask>>>,
    system_task_storage: Arc<RwLock<ApalisPostgresStorage<SystemTask>>>,
    cron_worker_id: String,
}

impl TaskService {
    pub async fn from_config_and_ctx(
        mut config: TaskConfig,
        ctx: Arc<dyn AppContextTrait>,
    ) -> RecorderResult<Self> {
        if config.subscriber_task_concurrency == 0 {
            config.subscriber_task_concurrency = default_subscriber_task_workers();
        };
        if config.system_task_concurrency == 0 {
            config.system_task_concurrency = default_system_task_workers();
        };

        let pool = ctx.db().get_postgres_connection_pool().clone();
        let subscriber_task_storage_config = Config::new(SUBSCRIBER_TASK_APALIS_NAME)
            .set_reenqueue_orphaned_after(config.subscriber_task_reenqueue_orphaned_after);
        let system_task_storage_config = Config::new(SYSTEM_TASK_APALIS_NAME)
            .set_reenqueue_orphaned_after(config.system_task_reenqueue_orphaned_after);
        let subscriber_task_storage =
            ApalisPostgresStorage::new_with_config(pool.clone(), subscriber_task_storage_config);
        let system_task_storage =
            ApalisPostgresStorage::new_with_config(pool, system_task_storage_config);

        Ok(Self {
            config,
            cron_worker_id: Uuid::now_v7().to_string(),
            ctx,
            subscriber_task_storage: Arc::new(RwLock::new(subscriber_task_storage)),
            system_task_storage: Arc::new(RwLock::new(system_task_storage)),
        })
    }

    async fn run_subscriber_task(
        job: SubscriberTask,
        data: Data<Arc<dyn AppContextTrait>>,
    ) -> RecorderResult<()> {
        let ctx = data.deref().clone();

        job.run_async(ctx).await
    }

    async fn run_system_task(
        job: SystemTask,
        data: Data<Arc<dyn AppContextTrait>>,
    ) -> RecorderResult<()> {
        let ctx = data.deref().clone();
        job.run_async(ctx).await
    }

    pub async fn retry_subscriber_task(&self, job_id: String) -> RecorderResult<()> {
        {
            let mut storage = self.subscriber_task_storage.write().await;
            let task_id =
                TaskId::from_str(&job_id).map_err(|err| RecorderError::InvalidTaskId {
                    message: err.to_string(),
                })?;
            let worker_id = WorkerId::new(SUBSCRIBER_TASK_APALIS_NAME);
            storage.retry(&worker_id, &task_id).await?;
        }
        Ok(())
    }

    pub async fn retry_system_task(&self, job_id: String) -> RecorderResult<()> {
        {
            let mut storage = self.system_task_storage.write().await;
            let task_id =
                TaskId::from_str(&job_id).map_err(|err| RecorderError::InvalidTaskId {
                    message: err.to_string(),
                })?;
            let worker_id = WorkerId::new(SYSTEM_TASK_APALIS_NAME);
            storage.retry(&worker_id, &task_id).await?;
        }
        Ok(())
    }

    pub async fn add_subscriber_task(
        &self,
        subscriber_task: SubscriberTask,
    ) -> RecorderResult<TaskId> {
        let task_id = {
            let mut storage = self.subscriber_task_storage.write().await;
            let sql_context = {
                let mut c = SqlContext::default();
                c.set_max_attempts(1);
                c
            };
            let request = Request::new_with_ctx(subscriber_task, sql_context);
            storage.push_request(request).await?.task_id
        };

        Ok(task_id)
    }

    pub async fn add_system_task(&self, system_task: SystemTask) -> RecorderResult<TaskId> {
        let task_id = {
            let mut storage = self.system_task_storage.write().await;
            let sql_context = {
                let mut c = SqlContext::default();
                c.set_max_attempts(1);
                c
            };
            let request = Request::new_with_ctx(system_task, sql_context);
            storage.push_request(request).await?.task_id
        };

        Ok(task_id)
    }

    pub async fn add_subscriber_task_cron(
        &self,
        cm: cron::ActiveModel,
    ) -> RecorderResult<cron::Model> {
        let db = self.ctx.db();
        let m = cm.insert(db).await?;
        Ok(m)
    }

    pub async fn add_system_task_cron(&self, cm: cron::ActiveModel) -> RecorderResult<cron::Model> {
        let db = self.ctx.db();
        let m = cm.insert(db).await?;
        Ok(m)
    }

    pub async fn run<F, Fut>(&self, shutdown_signal: Option<F>) -> RecorderResult<()>
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        tokio::try_join!(
            async {
                let monitor = self.setup_apalis_monitor().await?;
                if let Some(shutdown_signal) = shutdown_signal {
                    monitor
                        .run_with_signal(async move {
                            shutdown_signal().await;
                            tracing::info!("apalis shutting down...");
                            Ok(())
                        })
                        .await?;
                } else {
                    monitor.run().await?;
                }
                Ok::<_, RecorderError>(())
            },
            async {
                let listener = self.setup_apalis_listener().await?;
                tokio::task::spawn(async move {
                    if let Err(e) = listener.listen().await {
                        tracing::error!("Error listening to apalis: {e}");
                    }
                });
                Ok::<_, RecorderError>(())
            },
            async {
                let mut listener = self.setup_cron_due_listening().await?;
                let cron_worker_id = self.cron_worker_id.clone();
                let retry_duration = chrono::Duration::milliseconds(
                    self.config.cron_retry_duration.as_millis() as i64,
                );
                let cron_interval_duration = self.config.cron_interval_duration;
                listener.listen(CRON_DUE_EVENT).await?;
                tracing::debug!("Listening for cron due event...");

                tokio::task::spawn({
                    let ctx = self.ctx.clone();
                    async move {
                        if let Err(e) =
                            Self::listen_cron_due(listener, ctx, &cron_worker_id, retry_duration)
                                .await
                        {
                            tracing::error!("Error listening to cron due: {e}");
                        }
                    }
                });

                tokio::task::spawn({
                    let ctx = self.ctx.clone();
                    async move {
                        let mut interval = tokio::time::interval(cron_interval_duration);
                        loop {
                            interval.tick().await;
                            if let Err(e) = cron::Model::check_and_cleanup_expired_cron_locks(
                                ctx.as_ref(),
                                retry_duration,
                            )
                            .await
                            {
                                tracing::error!(
                                    "Error checking and cleaning up expired cron locks: {e}"
                                );
                            }
                            if let Err(e) =
                                cron::Model::check_and_trigger_due_crons(ctx.as_ref()).await
                            {
                                tracing::error!("Error checking and triggering due crons: {e}");
                            }
                        }
                    }
                });

                Ok::<_, RecorderError>(())
            }
        )?;
        Ok(())
    }

    async fn setup_apalis_monitor(&self) -> RecorderResult<Monitor> {
        let mut apalis_monitor = Monitor::new();

        {
            let subscriber_task_worker = WorkerBuilder::new(SUBSCRIBER_TASK_APALIS_NAME)
                .concurrency(self.config.subscriber_task_concurrency as usize)
                .catch_panic()
                .enable_tracing()
                .data(self.ctx.clone())
                .backend({
                    let storage = self.subscriber_task_storage.read().await;
                    storage.clone()
                })
                .build_fn(Self::run_subscriber_task);

            let system_task_worker = WorkerBuilder::new(SYSTEM_TASK_APALIS_NAME)
                .concurrency(self.config.system_task_concurrency as usize)
                .catch_panic()
                .enable_tracing()
                .data(self.ctx.clone())
                .backend(self.system_task_storage.read().await.clone())
                .build_fn(Self::run_system_task);

            apalis_monitor = apalis_monitor
                .register(subscriber_task_worker)
                .register(system_task_worker);
        }

        Ok(apalis_monitor)
    }

    async fn setup_apalis_listener(&self) -> RecorderResult<ApalisPgListen> {
        let pool = self.ctx.db().get_postgres_connection_pool().clone();
        let mut apalis_pg_listener = ApalisPgListen::new(pool).await?;

        {
            let mut subscriber_task_storage = self.subscriber_task_storage.write().await;
            apalis_pg_listener.subscribe_with(&mut subscriber_task_storage);
        }

        {
            let mut system_task_storage = self.system_task_storage.write().await;
            apalis_pg_listener.subscribe_with(&mut system_task_storage);
        }

        Ok(apalis_pg_listener)
    }

    async fn setup_cron_due_listening(&self) -> RecorderResult<PgListener> {
        let pool = self.ctx.db().get_postgres_connection_pool().clone();
        let listener = PgListener::connect_with(&pool).await?;
        tracing::debug!("Cron due listener connected to postgres");

        Ok(listener)
    }

    async fn listen_cron_due(
        mut listener: PgListener,
        ctx: Arc<dyn AppContextTrait>,
        worker_id: &str,
        retry_duration: chrono::Duration,
    ) -> RecorderResult<()> {
        loop {
            let notification = listener.recv().await?;
            tracing::debug!("Received cron due event: {:?}", notification);
            if let Err(e) = cron::Model::handle_cron_notification(
                ctx.as_ref(),
                notification,
                worker_id,
                retry_duration,
            )
            .await
            {
                tracing::error!("Error handling cron notification: {e}");
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use std::time::Duration;

    use rstest::{fixture, rstest};
    use sea_orm::ActiveValue;
    use tracing::Level;

    use super::*;
    use crate::{
        models::cron,
        task::EchoTask,
        test_utils::{
            app::{TestingAppContextConfig, TestingPreset},
            tracing::try_init_testing_tracing,
        },
    };

    #[fixture]
    fn before_each() {
        try_init_testing_tracing(Level::DEBUG);
    }

    #[rstest]
    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn test_cron_due_listening(before_each: ()) -> RecorderResult<()> {
        let preset = TestingPreset::default_with_config(
            TestingAppContextConfig::builder()
                .task_config(TaskConfig {
                    cron_interval_duration: Duration::from_secs(1),
                    ..Default::default()
                })
                .build(),
        )
        .await?;
        let app_ctx = preset.app_ctx;
        let task_service = app_ctx.task();

        let task_id = Uuid::now_v7().to_string();

        let echo_cron = cron::ActiveModel {
            cron_expr: ActiveValue::Set("*/1 * * * * *".to_string()),
            system_task_cron: ActiveValue::Set(Some(
                EchoTask::builder().task_id(task_id.clone()).build().into(),
            )),
            ..Default::default()
        };

        let _ = task_service
            .run(Some(async move || {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }))
            .await;

        // assert!(logs_contain(&format!(
        //     "EchoTask {task_id} start running at"
        // )));

        Ok(())
    }
}
