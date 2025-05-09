use sea_orm::{DeriveActiveEnum, DeriveDisplay, prelude::*};
use serde::{Deserialize, Serialize};

use super::mikan::MikanScrapeSeasonSubscriptionTask;

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    enum_name = "subscriber_task_type"
)]
#[serde(rename_all = "snake_case")]
pub enum SubscriberTaskType {
    #[sea_orm(string_value = "mikan_scrape_season_subscription")]
    MikanScrapeSeasonSubscription,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "task_type")]
pub enum SubscriberTaskPayload {
    #[serde(rename = "mikan_scrape_season_subscription")]
    MikanScrapeSeasonSubscription(MikanScrapeSeasonSubscriptionTask),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriberTask {
    pub id: i32,
    pub subscriber_id: i32,
    #[serde(flatten)]
    pub payload: SubscriberTaskPayload,
}
