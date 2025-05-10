use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(InputObject, Serialize, Deserialize)]
pub struct MikanScrapeSeasonSubscriptionInput {
    pub subscription_id: i32,
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct MikanScrapeSeasonSubscriptionOutput {
    pub task_id: i32,
}
