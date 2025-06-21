use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::mikan::{
        MikanBangumiSubscription, MikanSeasonSubscription, MikanSubscriberSubscription,
    },
    models::subscriptions::{self, SubscriptionTrait},
};

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, DeriveDisplay,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "subscription_category"
)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionCategory {
    #[sea_orm(string_value = "mikan_subscriber")]
    MikanSubscriber,
    #[sea_orm(string_value = "mikan_season")]
    MikanSeason,
    #[sea_orm(string_value = "mikan_bangumi")]
    MikanBangumi,
    #[sea_orm(string_value = "manual")]
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "category")]
pub enum Subscription {
    #[serde(rename = "mikan_subscriber")]
    MikanSubscriber(MikanSubscriberSubscription),
    #[serde(rename = "mikan_season")]
    MikanSeason(MikanSeasonSubscription),
    #[serde(rename = "mikan_bangumi")]
    MikanBangumi(MikanBangumiSubscription),
    #[serde(rename = "manual")]
    Manual,
}

impl Subscription {
    pub fn category(&self) -> SubscriptionCategory {
        match self {
            Self::MikanSubscriber(_) => SubscriptionCategory::MikanSubscriber,
            Self::MikanSeason(_) => SubscriptionCategory::MikanSeason,
            Self::MikanBangumi(_) => SubscriptionCategory::MikanBangumi,
            Self::Manual => SubscriptionCategory::Manual,
        }
    }
}

#[async_trait]
impl SubscriptionTrait for Subscription {
    fn get_subscriber_id(&self) -> i32 {
        match self {
            Self::MikanSubscriber(subscription) => subscription.get_subscriber_id(),
            Self::MikanSeason(subscription) => subscription.get_subscriber_id(),
            Self::MikanBangumi(subscription) => subscription.get_subscriber_id(),
            Self::Manual => unreachable!(),
        }
    }

    fn get_subscription_id(&self) -> i32 {
        match self {
            Self::MikanSubscriber(subscription) => subscription.get_subscription_id(),
            Self::MikanSeason(subscription) => subscription.get_subscription_id(),
            Self::MikanBangumi(subscription) => subscription.get_subscription_id(),
            Self::Manual => unreachable!(),
        }
    }

    async fn sync_feeds_incremental(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::MikanSubscriber(subscription) => subscription.sync_feeds_incremental(ctx).await,
            Self::MikanSeason(subscription) => subscription.sync_feeds_incremental(ctx).await,
            Self::MikanBangumi(subscription) => subscription.sync_feeds_incremental(ctx).await,
            Self::Manual => Ok(()),
        }
    }

    async fn sync_feeds_full(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::MikanSubscriber(subscription) => subscription.sync_feeds_full(ctx).await,
            Self::MikanSeason(subscription) => subscription.sync_feeds_full(ctx).await,
            Self::MikanBangumi(subscription) => subscription.sync_feeds_full(ctx).await,
            Self::Manual => Ok(()),
        }
    }

    async fn sync_sources(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        match self {
            Self::MikanSubscriber(subscription) => subscription.sync_sources(ctx).await,
            Self::MikanSeason(subscription) => subscription.sync_sources(ctx).await,
            Self::MikanBangumi(subscription) => subscription.sync_sources(ctx).await,
            Self::Manual => Ok(()),
        }
    }

    fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
        match model.category {
            SubscriptionCategory::MikanSubscriber => {
                MikanSubscriberSubscription::try_from_model(model).map(Self::MikanSubscriber)
            }
            SubscriptionCategory::MikanSeason => {
                MikanSeasonSubscription::try_from_model(model).map(Self::MikanSeason)
            }
            SubscriptionCategory::MikanBangumi => {
                MikanBangumiSubscription::try_from_model(model).map(Self::MikanBangumi)
            }
            SubscriptionCategory::Manual => Ok(Self::Manual),
        }
    }
}

impl TryFrom<&subscriptions::Model> for Subscription {
    type Error = RecorderError;

    fn try_from(model: &subscriptions::Model) -> Result<Self, Self::Error> {
        Self::try_from_model(model)
    }
}
