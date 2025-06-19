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
    task::{
        SUBSCRIBER_TASK_APALIS_NAME, SYSTEM_TASK_APALIS_NAME, SubscriberTask, TaskConfig,
        config::{default_subscriber_task_workers, default_system_task_workers},
        registry::SystemTask,
    },
};

pub struct TaskService {
    pub config: TaskConfig,
    ctx: Arc<dyn AppContextTrait>,
    subscriber_task_storage: Arc<RwLock<PostgresStorage<SubscriberTask>>>,
    system_task_storage: Arc<RwLock<PostgresStorage<SystemTask>>>,
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
        let subscriber_task_storage_config =
            Config::new(SUBSCRIBER_TASK_APALIS_NAME).set_keep_alive(config.subscriber_task_timeout);
        let system_task_storage_config =
            Config::new(SYSTEM_TASK_APALIS_NAME).set_keep_alive(config.system_task_timeout);
        let subscriber_task_storage =
            PostgresStorage::new_with_config(pool.clone(), subscriber_task_storage_config);
        let system_task_storage =
            PostgresStorage::new_with_config(pool, system_task_storage_config);

        Ok(Self {
            config,
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

        job.run(ctx).await
    }

    async fn run_system_task(
        job: SystemTask,
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

    pub async fn setup_monitor(&self) -> RecorderResult<Monitor> {
        let mut monitor = Monitor::new();

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

            monitor = monitor
                .register(subscriber_task_worker)
                .register(system_task_worker);
        }

        Ok(monitor)
    }

    pub async fn setup_listener(&self) -> RecorderResult<PgListen> {
        let pool = self.ctx.db().get_postgres_connection_pool().clone();
        let mut task_listener = PgListen::new(pool).await?;

        {
            let mut subscriber_task_storage = self.subscriber_task_storage.write().await;
            task_listener.subscribe_with(&mut subscriber_task_storage);
        }

        {
            let mut system_task_storage = self.system_task_storage.write().await;
            task_listener.subscribe_with(&mut system_task_storage);
        }

        Ok(task_listener)
    }
}
