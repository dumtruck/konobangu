use std::{ops::Deref, str::FromStr, sync::Arc};

use apalis::prelude::*;
use apalis_sql::{
    Config,
    context::SqlContext,
    postgres::{PgListen, PostgresStorage},
};
use tokio::sync::RwLock;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    task::{SUBSCRIBER_TASK_APALIS_NAME, SubscriberTask, TaskConfig},
};

pub struct TaskService {
    pub config: TaskConfig,
    ctx: Arc<dyn AppContextTrait>,
    subscriber_task_storage: Arc<RwLock<PostgresStorage<SubscriberTask>>>,
}

impl TaskService {
    pub async fn from_config_and_ctx(
        config: TaskConfig,
        ctx: Arc<dyn AppContextTrait>,
    ) -> RecorderResult<Self> {
        let pool = ctx.db().get_postgres_connection_pool().clone();
        let storage_config = Config::new(SUBSCRIBER_TASK_APALIS_NAME);
        let subscriber_task_storage = PostgresStorage::new_with_config(pool, storage_config);

        Ok(Self {
            config,
            ctx,
            subscriber_task_storage: Arc::new(RwLock::new(subscriber_task_storage)),
        })
    }

    async fn run_subscriber_task(
        job: SubscriberTask,
        data: Data<Arc<dyn AppContextTrait>>,
    ) -> RecorderResult<()> {
        let ctx = data.deref().clone();

        job.run(ctx).await
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

    pub async fn add_subscriber_task(
        &self,
        _subscriber_id: i32,
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

    pub async fn setup_monitor(&self) -> RecorderResult<Monitor> {
        let mut monitor = Monitor::new();

        {
            let subscriber_task_worker = WorkerBuilder::new(SUBSCRIBER_TASK_APALIS_NAME)
                .catch_panic()
                .enable_tracing()
                .data(self.ctx.clone())
                .backend({
                    let storage = self.subscriber_task_storage.read().await;
                    storage.clone()
                })
                .build_fn(Self::run_subscriber_task);

            monitor = monitor.register(subscriber_task_worker);
        }

        Ok(monitor)
    }

    pub async fn setup_listener(&self) -> RecorderResult<PgListen> {
        let pool = self.ctx.db().get_postgres_connection_pool().clone();
        let mut subscriber_task_listener = PgListen::new(pool).await?;

        {
            let mut subscriber_task_storage = self.subscriber_task_storage.write().await;
            subscriber_task_listener.subscribe_with(&mut subscriber_task_storage);
        }

        Ok(subscriber_task_listener)
    }
}
