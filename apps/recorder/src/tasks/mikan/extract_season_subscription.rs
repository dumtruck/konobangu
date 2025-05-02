use std::{ops::Deref, sync::Arc};

use apalis::prelude::*;
use apalis_sql::postgres::PostgresStorage;
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    extract::mikan::{
        MikanBangumiMeta, MikanSeasonStr, build_mikan_season_flow_url,
        scrape_mikan_bangumi_meta_list_from_season_flow_url,
    },
};

const TASK_NAME: &str = "mikan_extract_season_subscription";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractMikanSeasonSubscriptionTask {
    pub task_id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscription_id: i32,
    pub subscriber_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractMikanSeasonSubscriptionTaskResult {
    pub task_id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscription_id: i32,
    pub subscriber_id: i32,
    pub bangumi_meta_list: Vec<MikanBangumiMeta>,
}

pub async fn extract_mikan_season_subscription(
    job: ExtractMikanSeasonSubscriptionTask,
    data: Data<Arc<dyn AppContextTrait>>,
) -> RecorderResult<GoTo<ExtractMikanSeasonSubscriptionTaskResult>> {
    let ctx = data.deref();

    let mikan_client = ctx.mikan();
    let mikan_base_url = mikan_client.base_url();

    let mikan_season_flow_url =
        build_mikan_season_flow_url(mikan_base_url.clone(), job.year, job.season_str);

    let bangumi_meta_list = scrape_mikan_bangumi_meta_list_from_season_flow_url(
        mikan_client,
        ctx.clone(),
        mikan_season_flow_url,
        job.credential_id,
    )
    .await?;

    Ok(GoTo::Done(ExtractMikanSeasonSubscriptionTaskResult {
        bangumi_meta_list,
        credential_id: job.credential_id,
        season_str: job.season_str,
        subscriber_id: job.subscriber_id,
        subscription_id: job.subscription_id,
        task_id: job.task_id,
        year: job.year,
    }))
}

pub fn register_extract_mikan_season_subscription_task(
    monitor: Monitor,
    ctx: Arc<dyn AppContextTrait>,
) -> RecorderResult<(Monitor, PostgresStorage<StepRequest<serde_json::Value>>)> {
    let pool = ctx.db().get_postgres_connection_pool().clone();
    let storage = PostgresStorage::new(pool);

    let steps = StepBuilder::new().step_fn(extract_mikan_season_subscription);

    let worker = WorkerBuilder::new(TASK_NAME)
        .catch_panic()
        .enable_tracing()
        .data(ctx)
        .backend(storage.clone())
        .build_stepped(steps);

    Ok((monitor.register(worker), storage))
}
