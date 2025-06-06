mod subscription;
use std::sync::Arc;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
pub use subscription::{
    SyncOneSubscriptionFeedsFullTask, SyncOneSubscriptionFeedsIncrementalTask,
    SyncOneSubscriptionSourcesTask,
};

use super::SubscriberAsyncTaskTrait;
use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
};

#[derive(async_graphql::Enum, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum SubscriberTaskType {
    #[serde(rename = "sync_one_subscription_feeds_incremental")]
    #[graphql(name = "sync_one_subscription_feeds_incremental")]
    SyncOneSubscriptionFeedsIncremental,
    #[serde(rename = "sync_one_subscription_feeds_full")]
    #[graphql(name = "sync_one_subscription_feeds_full")]
    SyncOneSubscriptionFeedsFull,
    #[serde(rename = "sync_one_subscription_sources")]
    #[graphql(name = "sync_one_subscription_sources")]
    SyncOneSubscriptionSources,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "task_type")]
pub enum SubscriberTaskPayload {
    #[serde(rename = "sync_one_subscription_feeds_incremental")]
    SyncOneSubscriptionFeedsIncremental(SyncOneSubscriptionFeedsIncrementalTask),
    #[serde(rename = "sync_one_subscription_feeds_full")]
    SyncOneSubscriptionFeedsFull(SyncOneSubscriptionFeedsFullTask),
    #[serde(rename = "sync_one_subscription_sources")]
    SyncOneSubscriptionSources(SyncOneSubscriptionSourcesTask),
}

impl SubscriberTaskPayload {
    pub async fn run(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::SyncOneSubscriptionFeedsIncremental(task) => task.run(ctx).await,
            Self::SyncOneSubscriptionFeedsFull(task) => task.run(ctx).await,
            Self::SyncOneSubscriptionSources(task) => task.run(ctx).await,
        }
    }

    pub fn task_type(&self) -> SubscriberTaskType {
        match self {
            Self::SyncOneSubscriptionFeedsIncremental(_) => {
                SubscriberTaskType::SyncOneSubscriptionFeedsIncremental
            }
            Self::SyncOneSubscriptionFeedsFull(_) => {
                SubscriberTaskType::SyncOneSubscriptionFeedsFull
            }
            Self::SyncOneSubscriptionSources(_) => SubscriberTaskType::SyncOneSubscriptionSources,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, FromJsonQueryResult)]
pub struct SubscriberTask {
    pub subscriber_id: i32,
    #[serde(flatten)]
    pub payload: SubscriberTaskPayload,
}
