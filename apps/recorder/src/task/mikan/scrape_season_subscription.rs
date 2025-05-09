use std::sync::Arc;

use futures::Stream;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    extract::mikan::{
        MikanBangumiMeta, MikanSeasonStr, build_mikan_season_flow_url,
        scrape_mikan_bangumi_meta_stream_from_season_flow_url,
    },
    task::SubscriberStreamTaskTrait,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, FromJsonQueryResult)]
pub struct MikanScrapeSeasonSubscriptionTask {
    pub task_id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriberStreamTaskTrait for MikanScrapeSeasonSubscriptionTask {
    type Yield = MikanBangumiMeta;

    fn run_stream(
        self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = RecorderResult<Self::Yield>> {
        let mikan_base_url = ctx.mikan().base_url().clone();

        let mikan_season_flow_url =
            build_mikan_season_flow_url(mikan_base_url, self.year, self.season_str);

        scrape_mikan_bangumi_meta_stream_from_season_flow_url(
            ctx.clone(),
            mikan_season_flow_url,
            self.credential_id,
        )
    }
}
