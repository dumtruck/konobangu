use std::sync::Arc;

use async_graphql::{Context, InputObject, Object, Result as GraphQLResult, SimpleObject};

use crate::{
    app::AppContextTrait,
    auth::AuthUserInfo,
    models::subscriptions::{self, SubscriptionTrait},
    task::SubscriberTaskPayload,
};

pub struct SubscriptionMutation;

#[derive(InputObject)]
struct SyncOneSubscriptionFilterInput {
    pub subscription_id: i32,
}

#[derive(SimpleObject)]
struct SyncOneSubscriptionTaskOutput {
    pub task_id: String,
}

#[Object]
impl SubscriptionMutation {
    async fn sync_one_subscription_feeds_incremental(
        &self,
        ctx: &Context<'_>,
        input: SyncOneSubscriptionFilterInput,
    ) -> GraphQLResult<SyncOneSubscriptionTaskOutput> {
        let auth_user_info = ctx.data::<AuthUserInfo>()?;

        let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;
        let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

        let subscription_model = subscriptions::Model::find_by_id_and_subscriber_id(
            app_ctx.as_ref(),
            input.subscription_id,
            subscriber_id,
        )
        .await?;

        let subscription = subscriptions::Subscription::try_from_model(&subscription_model)?;

        let task_service = app_ctx.task();

        let task_id = task_service
            .add_subscriber_task(
                auth_user_info.subscriber_auth.subscriber_id,
                SubscriberTaskPayload::SyncOneSubscriptionFeedsIncremental(subscription.into()),
            )
            .await?;

        Ok(SyncOneSubscriptionTaskOutput {
            task_id: task_id.to_string(),
        })
    }

    async fn sync_one_subscription_feeds_full(
        &self,
        ctx: &Context<'_>,
        input: SyncOneSubscriptionFilterInput,
    ) -> GraphQLResult<SyncOneSubscriptionTaskOutput> {
        let auth_user_info = ctx.data::<AuthUserInfo>()?;

        let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;
        let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

        let subscription_model = subscriptions::Model::find_by_id_and_subscriber_id(
            app_ctx.as_ref(),
            input.subscription_id,
            subscriber_id,
        )
        .await?;

        let subscription = subscriptions::Subscription::try_from_model(&subscription_model)?;

        let task_service = app_ctx.task();

        let task_id = task_service
            .add_subscriber_task(
                auth_user_info.subscriber_auth.subscriber_id,
                SubscriberTaskPayload::SyncOneSubscriptionFeedsFull(subscription.into()),
            )
            .await?;

        Ok(SyncOneSubscriptionTaskOutput {
            task_id: task_id.to_string(),
        })
    }

    async fn sync_one_subscription_sources(
        &self,
        ctx: &Context<'_>,
        input: SyncOneSubscriptionFilterInput,
    ) -> GraphQLResult<SyncOneSubscriptionTaskOutput> {
        let auth_user_info = ctx.data::<AuthUserInfo>()?;

        let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;
        let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

        let subscription_model = subscriptions::Model::find_by_id_and_subscriber_id(
            app_ctx.as_ref(),
            input.subscription_id,
            subscriber_id,
        )
        .await?;

        let subscription = subscriptions::Subscription::try_from_model(&subscription_model)?;

        let task_service = app_ctx.task();

        let task_id = task_service
            .add_subscriber_task(
                auth_user_info.subscriber_auth.subscriber_id,
                SubscriberTaskPayload::SyncOneSubscriptionSources(subscription.into()),
            )
            .await?;

        Ok(SyncOneSubscriptionTaskOutput {
            task_id: task_id.to_string(),
        })
    }
}
