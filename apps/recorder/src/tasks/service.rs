use std::{fmt::Debug, sync::Arc};

use apalis::prelude::*;
use apalis_sql::postgres::PostgresStorage;
use tokio::sync::Mutex;

use super::{TaskConfig, mikan::register_extract_mikan_season_subscription_task};
use crate::{app::AppContextTrait, errors::RecorderResult};

pub struct TaskService {
    config: TaskConfig,
    #[allow(dead_code)]
    monitor: Arc<Mutex<Monitor>>,
    pub extract_mikan_season_subscription_task_storage:
        PostgresStorage<StepRequest<serde_json::Value>>,
}

impl TaskService {
    pub async fn from_config_and_ctx(
        config: TaskConfig,
        ctx: Arc<dyn AppContextTrait>,
    ) -> RecorderResult<Self> {
        let monitor = Monitor::new();
        let (monitor, extract_mikan_season_subscription_task_storage) =
            register_extract_mikan_season_subscription_task(monitor, ctx.clone())?;

        Ok(Self {
            config,
            monitor: Arc::new(Mutex::new(monitor)),
            extract_mikan_season_subscription_task_storage,
        })
    }
}

impl Debug for TaskService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskService")
            .field("config", &self.config)
            .finish()
    }
}
