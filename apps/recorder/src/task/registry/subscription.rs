use std::sync::Arc;

use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    models::subscriptions::{self, SubscriptionTrait},
    task::AsyncTaskTrait,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncOneSubscriptionFeedsIncrementalTask(pub subscriptions::Subscription);

impl From<subscriptions::Subscription> for SyncOneSubscriptionFeedsIncrementalTask {
    fn from(subscription: subscriptions::Subscription) -> Self {
        Self(subscription)
    }
}

#[async_trait::async_trait]
impl AsyncTaskTrait for SyncOneSubscriptionFeedsIncrementalTask {
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        self.0.sync_feeds_incremental(ctx).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncOneSubscriptionFeedsFullTask(pub subscriptions::Subscription);

impl From<subscriptions::Subscription> for SyncOneSubscriptionFeedsFullTask {
    fn from(subscription: subscriptions::Subscription) -> Self {
        Self(subscription)
    }
}

#[async_trait::async_trait]
impl AsyncTaskTrait for SyncOneSubscriptionFeedsFullTask {
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        self.0.sync_feeds_full(ctx).await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncOneSubscriptionSourcesTask(pub subscriptions::Subscription);

#[async_trait::async_trait]
impl AsyncTaskTrait for SyncOneSubscriptionSourcesTask {
    async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        self.0.sync_sources(ctx).await?;
        Ok(())
    }
}

impl From<subscriptions::Subscription> for SyncOneSubscriptionSourcesTask {
    fn from(subscription: subscriptions::Subscription) -> Self {
        Self(subscription)
    }
}
