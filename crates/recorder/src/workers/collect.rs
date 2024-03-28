use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::models::bangumi;

pub struct CollectHistoryEpisodesWorker {
    pub ctx: AppContext,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CollectHistoryEpisodesWorkerArgs {
    CollectFromBangumiEntity(),
}

impl CollectHistoryEpisodesWorker {
    pub async fn collect_history_episodes(bangumi: &bangumi::Model, _only_season: bool) {
        info!(
            "Start collecting {} season {}...",
            bangumi.official_title, bangumi.season
        );
        todo!()
    }
}

impl worker::AppWorker<CollectHistoryEpisodesWorkerArgs> for CollectHistoryEpisodesWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
}

#[async_trait]
impl worker::Worker<CollectHistoryEpisodesWorkerArgs> for CollectHistoryEpisodesWorker {
    async fn perform(&self, _args: CollectHistoryEpisodesWorkerArgs) -> worker::Result<()> {
        println!("================================================");

        let db = &self.ctx.db;

        println!("================================================");
        Ok(())
    }
}
