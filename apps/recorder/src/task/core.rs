use std::sync::Arc;

use futures::Stream;
use serde::{Serialize, de::DeserializeOwned};

use crate::{app::AppContextTrait, errors::RecorderResult};

pub const SYSTEM_TASK_APALIS_NAME: &str = "system_task";
pub const SUBSCRIBER_TASK_APALIS_NAME: &str = "subscriber_task";

#[async_trait::async_trait]
pub trait AsyncTaskTrait: Serialize + DeserializeOwned + Sized {
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()>;

    async fn run(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        self.run_async(ctx).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait StreamTaskTrait: Serialize + DeserializeOwned + Sized {
    type Yield: Serialize + DeserializeOwned + Send;

    fn run_stream(
        self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = RecorderResult<Self::Yield>> + Send;

    async fn run(self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        unimplemented!()
    }
}
