use std::sync::Arc;

use futures::{Stream, TryStreamExt, pin_mut};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    models::subscriber_tasks::{self, SubscriberTaskErrorSnapshot},
};

pub const SUBSCRIBER_TASK_APALIS_NAME: &str = "subscriber_task";

#[async_trait::async_trait]
pub trait SubscriberAsyncTaskTrait: Serialize + DeserializeOwned + Sized {
    type Result: Serialize + DeserializeOwned + Send;

    async fn run_async(
        self,
        ctx: Arc<dyn AppContextTrait>,
        id: i32,
    ) -> RecorderResult<Self::Result>;

    async fn run(self, ctx: Arc<dyn AppContextTrait>, id: i32) -> RecorderResult<()> {
        match self.run_async(ctx.clone(), id).await {
            Ok(result) => {
                subscriber_tasks::Model::update_result(ctx, id, result).await?;
            }
            Err(e) => {
                let error_snapshot = SubscriberTaskErrorSnapshot {
                    message: e.to_string(),
                };

                subscriber_tasks::Model::update_error(ctx, id, error_snapshot).await?;

                return Err(e);
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait SubscriberStreamTaskTrait: Serialize + DeserializeOwned + Sized {
    type Yield: Serialize + DeserializeOwned + Send;

    fn run_stream(
        self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = RecorderResult<Self::Yield>> + Send;

    async fn run(self, ctx: Arc<dyn AppContextTrait>, id: i32) -> RecorderResult<()> {
        let stream = self.run_stream(ctx.clone());

        pin_mut!(stream);

        loop {
            match stream.try_next().await {
                Ok(Some(result)) => {
                    subscriber_tasks::Model::append_yield(ctx.clone(), id, result).await?;
                }
                Ok(None) => {
                    subscriber_tasks::Model::update_result(ctx, id, ()).await?;
                    break;
                }
                Err(e) => {
                    let error_snapshot = SubscriberTaskErrorSnapshot {
                        message: e.to_string(),
                    };

                    subscriber_tasks::Model::update_error(ctx, id, error_snapshot).await?;

                    return Err(e);
                }
            }
        }

        Ok(())
    }
}
