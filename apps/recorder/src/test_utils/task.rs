use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    task::{AsyncTaskTrait, TaskConfig, TaskService, register_system_task_type},
};

register_system_task_type! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct TestSystemTask {
        pub task_id: String,
    }
}

#[async_trait::async_trait]
impl AsyncTaskTrait for TestSystemTask {
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let storage = ctx.storage();

        storage
            .write(
                storage.build_test_path(self.task_id),
                serde_json::json!({ "exec_time": Utc::now().timestamp_millis() })
                    .to_string()
                    .into(),
            )
            .await?;

        Ok(())
    }
}

pub async fn build_testing_task_service(
    ctx: Arc<dyn AppContextTrait>,
) -> RecorderResult<TaskService> {
    let config = TaskConfig::default();
    let task_service = TaskService::from_config_and_ctx(config, ctx).await?;

    Ok(task_service)
}
