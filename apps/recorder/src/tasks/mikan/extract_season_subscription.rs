use std::{ops::Deref, sync::Arc};

use apalis::prelude::*;
use apalis_sql::postgres::PostgresStorage;
use fetch::fetch_html;
use serde::{Deserialize, Serialize};
use snafu::OptionExt;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::mikan::{
        MikanBangumiMeta, MikanSeasonStr, build_mikan_season_flow_url,
        extract_mikan_bangumi_indices_meta_from_season_flow_fragment,
        web_extract::{
            MikanBangumiIndexMeta, build_mikan_bangumi_expand_subscribed_fragment_url,
            extract_mikan_bangumi_meta_from_expand_subscribed_fragment,
        },
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
pub struct ExtractMikanSeasonSubscriptionFansubsTask {
    pub task_id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscription_id: i32,
    pub subscriber_id: i32,
    pub bangumi_indices: Vec<MikanBangumiIndexMeta>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractMikanSeasonSubscriptionTaskResult {
    pub task_id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscription_id: i32,
    pub subscriber_id: i32,
    pub bangumi_metas: Vec<MikanBangumiMeta>,
}

pub async fn extract_mikan_season_subscription(
    job: ExtractMikanSeasonSubscriptionTask,
    data: Data<Arc<dyn AppContextTrait>>,
) -> RecorderResult<GoTo<ExtractMikanSeasonSubscriptionFansubsTask>> {
    let ctx = data.deref();

    let mikan_client = ctx
        .mikan()
        .fork_with_credential(ctx.clone(), Some(job.credential_id))
        .await?;

    let mikan_base_url = mikan_client.base_url().clone();

    let season_flow_fragment_url =
        build_mikan_season_flow_url(mikan_base_url.clone(), job.year, job.season_str);

    let season_flow_fragment = fetch_html(&mikan_client, season_flow_fragment_url.clone()).await?;

    let mut bangumi_indices = extract_mikan_bangumi_indices_meta_from_season_flow_fragment(
        &season_flow_fragment,
        mikan_base_url.clone(),
    );

    if bangumi_indices.is_empty() && !mikan_client.has_login().await? {
        mikan_client.login().await?;
        let season_flow_fragment =
            fetch_html(&mikan_client, season_flow_fragment_url.clone()).await?;
        bangumi_indices = extract_mikan_bangumi_indices_meta_from_season_flow_fragment(
            &season_flow_fragment,
            mikan_base_url.clone(),
        );
    }

    Ok(GoTo::Next(ExtractMikanSeasonSubscriptionFansubsTask {
        task_id: job.task_id,
        year: job.year,
        season_str: job.season_str,
        credential_id: job.credential_id,
        subscription_id: job.subscription_id,
        subscriber_id: job.subscriber_id,
        bangumi_indices,
    }))
}

pub async fn extract_mikan_season_subscription_fansubs(
    job: ExtractMikanSeasonSubscriptionFansubsTask,
    data: Data<Arc<dyn AppContextTrait>>,
) -> RecorderResult<GoTo<ExtractMikanSeasonSubscriptionTaskResult>> {
    let ctx = data.deref();

    let mikan_client = ctx
        .mikan()
        .fork_with_credential(ctx.clone(), Some(job.credential_id))
        .await?;

    let bangumi_indices = job.bangumi_indices;

    let mut bangumi_metas = vec![];

    let mikan_base_url = mikan_client.base_url().clone();

    for bangumi_index in bangumi_indices {
        let bangumi_title = bangumi_index.bangumi_title.clone();
        let bangumi_expand_subscribed_fragment_url =
            build_mikan_bangumi_expand_subscribed_fragment_url(
                mikan_base_url.clone(),
                &bangumi_index.mikan_bangumi_id,
            );
        let bangumi_expand_subscribed_fragment =
            fetch_html(&mikan_client, bangumi_expand_subscribed_fragment_url).await?;

        let bangumi_meta = extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
            bangumi_index,
            &bangumi_expand_subscribed_fragment,
            mikan_base_url.clone(),
        )
        .with_whatever_context::<_, String, RecorderError>(|| {
            format!(
                "failed to extract mikan bangumi fansub of title = {}",
                bangumi_title
            )
        })?;

        bangumi_metas.push(bangumi_meta);
    }

    Ok(GoTo::Done(ExtractMikanSeasonSubscriptionTaskResult {
        bangumi_metas,
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

    let steps = StepBuilder::new()
        .step_fn(extract_mikan_season_subscription)
        .step_fn(extract_mikan_season_subscription_fansubs);

    let worker = WorkerBuilder::new(TASK_NAME)
        .catch_panic()
        .enable_tracing()
        .data(ctx)
        .backend(storage.clone())
        .build_stepped(steps);

    Ok((monitor.register(worker), storage))
}
