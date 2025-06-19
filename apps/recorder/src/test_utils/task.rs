use std::sync::Arc;

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    task::{TaskConfig, TaskService},
};

pub async fn build_testing_task_service(
    ctx: Arc<dyn AppContextTrait>,
) -> RecorderResult<TaskService> {
    let config = TaskConfig::default();
    let task_service = TaskService::from_config_and_ctx(config, ctx).await?;
    Ok(task_service)
}
