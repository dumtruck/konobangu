mod subscription;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub use subscription::{SyncOneSubscriptionFeedsTask, SyncOneSubscriptionSourcesTask};

use super::SubscriberAsyncTaskTrait;
use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "task_type")]
pub enum SubscriberTaskPayload {
    #[serde(rename = "sync_one_subscription_feeds")]
    SyncOneSubscriptionFeeds(SyncOneSubscriptionFeedsTask),
    #[serde(rename = "sync_one_subscription_sources")]
    SyncOneSubscriptionSources(SyncOneSubscriptionSourcesTask),
}

impl SubscriberTaskPayload {
    pub async fn run(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::SyncOneSubscriptionFeeds(task) => task.run(ctx).await,
            Self::SyncOneSubscriptionSources(task) => task.run(ctx).await,
        }
    }
}

impl TryFrom<&SubscriberTaskPayload> for serde_json::Value {
    type Error = RecorderError;

    fn try_from(value: &SubscriberTaskPayload) -> Result<Self, Self::Error> {
        let json_value = serde_json::to_value(value)?;
        Ok(match json_value {
            serde_json::Value::Object(mut map) => {
                map.remove("task_type");
                serde_json::Value::Object(map)
            }
            _ => {
                unreachable!("subscriber task payload must be an json object");
            }
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriberTask {
    pub subscriber_id: i32,
    #[serde(flatten)]
    pub payload: SubscriberTaskPayload,
}
