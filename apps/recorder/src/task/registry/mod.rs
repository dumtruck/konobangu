mod media;
mod subscription;
use std::sync::Arc;

pub use media::OptimizeImageTask;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
pub use subscription::{
    SyncOneSubscriptionFeedsFullTask, SyncOneSubscriptionFeedsIncrementalTask,
    SyncOneSubscriptionSourcesTask,
};

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    models::subscriptions::SubscriptionTrait,
    task::AsyncTaskTrait,
};

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Copy,
    DeriveActiveEnum,
    DeriveDisplay,
    EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SubscriberTaskType {
    #[serde(rename = "sync_one_subscription_feeds_incremental")]
    #[sea_orm(string_value = "sync_one_subscription_feeds_incremental")]
    SyncOneSubscriptionFeedsIncremental,
    #[serde(rename = "sync_one_subscription_feeds_full")]
    #[sea_orm(string_value = "sync_one_subscription_feeds_full")]
    SyncOneSubscriptionFeedsFull,
    #[serde(rename = "sync_one_subscription_sources")]
    #[sea_orm(string_value = "sync_one_subscription_sources")]
    SyncOneSubscriptionSources,
}

impl TryFrom<&SubscriberTask> for serde_json::Value {
    type Error = RecorderError;

    fn try_from(value: &SubscriberTask) -> Result<Self, Self::Error> {
        let json_value = serde_json::to_value(value)?;
        Ok(match json_value {
            serde_json::Value::Object(mut map) => {
                map.remove("task_type");
                serde_json::Value::Object(map)
            }
            _ => {
                unreachable!("subscriber task must be an json object");
            }
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, FromJsonQueryResult)]
#[serde(tag = "task_type")]
pub enum SubscriberTask {
    #[serde(rename = "sync_one_subscription_feeds_incremental")]
    SyncOneSubscriptionFeedsIncremental(SyncOneSubscriptionFeedsIncrementalTask),
    #[serde(rename = "sync_one_subscription_feeds_full")]
    SyncOneSubscriptionFeedsFull(SyncOneSubscriptionFeedsFullTask),
    #[serde(rename = "sync_one_subscription_sources")]
    SyncOneSubscriptionSources(SyncOneSubscriptionSourcesTask),
}

impl SubscriberTask {
    pub fn get_subscriber_id(&self) -> i32 {
        match self {
            Self::SyncOneSubscriptionFeedsIncremental(task) => task.0.get_subscriber_id(),
            Self::SyncOneSubscriptionFeedsFull(task) => task.0.get_subscriber_id(),
            Self::SyncOneSubscriptionSources(task) => task.0.get_subscriber_id(),
        }
    }

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

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Copy,
    DeriveActiveEnum,
    DeriveDisplay,
    EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SystemTaskType {
    #[serde(rename = "optimize_image")]
    #[sea_orm(string_value = "optimize_image")]
    OptimizeImage,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromJsonQueryResult)]
pub enum SystemTask {
    #[serde(rename = "optimize_image")]
    OptimizeImage(OptimizeImageTask),
}

impl SystemTask {
    pub async fn run(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::OptimizeImage(task) => task.run(ctx).await,
        }
    }
}
