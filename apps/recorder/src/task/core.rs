use std::sync::Arc;

use async_trait::async_trait;
use futures::{Stream, StreamExt, pin_mut};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{app::AppContextTrait, errors::RecorderResult};

pub const SYSTEM_TASK_APALIS_NAME: &str = "system_task";
pub const SUBSCRIBER_TASK_APALIS_NAME: &str = "subscriber_task";

#[async_trait]
pub trait AsyncTaskTrait: Serialize + DeserializeOwned + Sized {
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()>;
}

pub trait StreamTaskTrait {
    type Yield: Serialize + DeserializeOwned + Send;

    fn run_stream(
        self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = RecorderResult<Self::Yield>> + Send;
}

#[async_trait]
impl<T> AsyncTaskTrait for T
where
    T: StreamTaskTrait + Serialize + DeserializeOwned + Sized + Send,
{
    async fn run_async(self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let s = self.run_stream(_ctx);

        pin_mut!(s);

        while let Some(item) = s.next().await {
            item?;
        }

        Ok(())
    }
}

pub trait SubscriberTaskTrait: AsyncTaskTrait {
    fn get_subscriber_id(&self) -> i32;

    fn get_cron_id(&self) -> Option<i32>;
}

pub trait SystemTaskTrait: AsyncTaskTrait {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct SubscriberTaskBase {
    pub subscriber_id: i32,
    pub cron_id: Option<i32>,
}
