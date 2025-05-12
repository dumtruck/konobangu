use std::{ops::Deref, sync::Arc};

use apalis::prelude::*;
use apalis_sql::postgres::PostgresStorage;
use tokio::sync::RwLock;

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    task::{SUBSCRIBER_TASK_APALIS_NAME, SubscriberTask, SubscriberTaskPayload, TaskConfig},
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
        let subscriber_task_storage = Arc::new(RwLock::new(PostgresStorage::new(pool)));

        Ok(Self {
            config,
            ctx,
            subscriber_task_storage,
        })
    }

    async fn run_subscriber_task(
        job: SubscriberTask,
        data: Data<Arc<dyn AppContextTrait>>,
    ) -> RecorderResult<()> {
        let ctx = data.deref().clone();

        job.payload.run(ctx).await
    }

    pub async fn add_subscriber_task(
        &self,
        subscriber_id: i32,
        task_payload: SubscriberTaskPayload,
    ) -> RecorderResult<TaskId> {
        let subscriber_task = SubscriberTask {
            subscriber_id,
            payload: task_payload,
        };

        let task_id = {
            let mut storage = self.subscriber_task_storage.write().await;
            storage.push(subscriber_task).await?.task_id
        };

        Ok(task_id)
    }

    pub async fn setup(&self) -> RecorderResult<()> {
        let monitor = Monitor::new();
        let worker = WorkerBuilder::new(SUBSCRIBER_TASK_APALIS_NAME)
            .catch_panic()
            .enable_tracing()
            .data(self.ctx.clone())
            .backend({
                let storage = self.subscriber_task_storage.read().await;
                storage.clone()
            })
            .build_fn(Self::run_subscriber_task);

        let monitor = monitor.register(worker);

        monitor.run().await?;

        Ok(())
    }
}
