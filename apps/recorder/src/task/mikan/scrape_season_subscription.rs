use std::sync::Arc;

use futures::Stream;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    extract::mikan::{MikanBangumiMeta, MikanSeasonStr, MikanSeasonSubscription},
    task::SubscriberStreamTaskTrait,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, FromJsonQueryResult)]
pub struct MikanScrapeSeasonSubscriptionTask {
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
}

#[async_trait::async_trait]
impl SubscriberStreamTaskTrait for MikanScrapeSeasonSubscriptionTask {
    type Yield = MikanBangumiMeta;

    fn run_stream(
        self,
        ctx: Arc<dyn AppContextTrait>,
        id: i32,
    ) -> impl Stream<Item = RecorderResult<Self::Yield>> {
        let task = Arc::new(MikanSeasonSubscription {
            id,
            year: self.year,
            season_str: self.season_str,
            credential_id: self.credential_id,
        });

        task.pull_bangumi_meta_stream(ctx)
    }
}
