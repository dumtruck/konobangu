mod scrape_season_subscription;

use std::sync::Arc;

use async_graphql::{Context, Object, Result as GraphQLResult};
use snafu::FromString;

use crate::{
    app::AppContextTrait,
    auth::AuthUserInfo,
    errors::RecorderError,
    graphql::mikan::scrape_season_subscription::{
        MikanScrapeSeasonSubscriptionInput, MikanScrapeSeasonSubscriptionOutput,
    },
    models::{
        subscriber_tasks,
        subscriptions::{self, SubscriptionCategory},
    },
    task::{SubscriberTaskPayload, mikan::MikanScrapeSeasonSubscriptionTask},
};

struct MikanQuery;

struct MikanMutation;

#[Object]
impl MikanMutation {
    async fn mikan_scrape_season_subscription(
        &self,
        ctx: &Context<'_>,
        input: MikanScrapeSeasonSubscriptionInput,
    ) -> GraphQLResult<MikanScrapeSeasonSubscriptionOutput> {
        let auth_user = ctx.data::<AuthUserInfo>()?;
        let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

        let subscription =
            subscriptions::Model::find_by_id(app_ctx.as_ref(), input.subscription_id)
                .await?
                .ok_or_else(|| RecorderError::DbError {
                    source: sea_orm::DbErr::RecordNotFound(String::from("subscription not found")),
                })?;

        if subscription.category != SubscriptionCategory::MikanSeason {
            Err(RecorderError::without_source(
                "subscription must be a mikan season subscription".to_string(),
            ))?;
        }

        let credential_id = subscription.credential_id.ok_or_else(|| {
            RecorderError::without_source("subscription must have a credential".to_string())
        })?;

        let task = subscriber_tasks::Model::add_subscriber_task(
            app_ctx.clone(),
            auth_user.subscriber_auth.subscriber_id,
            SubscriberTaskPayload::MikanScrapeSeasonSubscription(todo!()),
        )
        .await?;

        Ok(MikanScrapeSeasonSubscriptionOutput { task_id: 1 })
    }
}

struct MikanSubscription;
