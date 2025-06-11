use std::sync::Arc;

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputObject, InputValue, Object, TypeRef,
};
use seaography::Builder as SeaographyBuilder;
use serde::{Deserialize, Serialize};
use util_derive::DynamicGraphql;

use crate::{
    app::AppContextTrait,
    auth::AuthUserInfo,
    models::subscriptions::{self, SubscriptionTrait},
    task::SubscriberTask,
};

#[derive(DynamicGraphql, Serialize, Deserialize, Clone, Debug)]
struct SyncOneSubscriptionFilterInput {
    pub id: i32,
}

impl SyncOneSubscriptionFilterInput {
    fn input_type_name() -> &'static str {
        "SyncOneSubscriptionFilterInput"
    }

    fn arg_name() -> &'static str {
        "filter"
    }

    fn generate_input_object() -> InputObject {
        InputObject::new(Self::input_type_name())
            .description("The input of the subscriptionSyncOne series of mutations")
            .field(InputValue::new(
                SyncOneSubscriptionFilterInputFieldEnum::Id.as_str(),
                TypeRef::named_nn(TypeRef::INT),
            ))
    }
}

#[derive(DynamicGraphql, Serialize, Deserialize, Clone, Debug)]
pub struct SyncOneSubscriptionInfo {
    pub task_id: String,
}

impl SyncOneSubscriptionInfo {
    fn object_type_name() -> &'static str {
        "SyncOneSubscriptionInfo"
    }

    fn generate_output_object() -> Object {
        Object::new(Self::object_type_name())
            .description("The output of the subscriptionSyncOne series of mutations")
            .field(Field::new(
                SyncOneSubscriptionInfoFieldEnum::TaskId,
                TypeRef::named_nn(TypeRef::STRING),
                move |ctx| {
                    FieldFuture::new(async move {
                        let subscription_info = ctx.parent_value.try_downcast_ref::<Self>()?;
                        Ok(Some(async_graphql::Value::from(
                            subscription_info.task_id.as_str(),
                        )))
                    })
                },
            ))
    }
}

pub fn register_subscriptions_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.schema = builder
        .schema
        .register(SyncOneSubscriptionFilterInput::generate_input_object());
    builder.schema = builder
        .schema
        .register(SyncOneSubscriptionInfo::generate_output_object());

    builder.mutations.push(
        Field::new(
            "subscriptionSyncOneFeedsIncremental",
            TypeRef::named_nn(SyncOneSubscriptionInfo::object_type_name()),
            move |ctx| {
                FieldFuture::new(async move {
                    let auth_user_info = ctx.data::<AuthUserInfo>()?;

                    let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;
                    let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

                    let filter_input: SyncOneSubscriptionFilterInput = ctx
                        .args
                        .get(SyncOneSubscriptionFilterInput::arg_name())
                        .unwrap()
                        .deserialize()?;

                    let subscription_model = subscriptions::Model::find_by_id_and_subscriber_id(
                        app_ctx.as_ref(),
                        filter_input.id,
                        subscriber_id,
                    )
                    .await?;

                    let subscription =
                        subscriptions::Subscription::try_from_model(&subscription_model)?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            auth_user_info.subscriber_auth.subscriber_id,
                            SubscriberTask::SyncOneSubscriptionFeedsIncremental(
                                subscription.into(),
                            ),
                        )
                        .await?;

                    Ok(Some(FieldValue::owned_any(SyncOneSubscriptionInfo {
                        task_id: task_id.to_string(),
                    })))
                })
            },
        )
        .argument(InputValue::new(
            SyncOneSubscriptionFilterInput::arg_name(),
            TypeRef::named_nn(SyncOneSubscriptionFilterInput::input_type_name()),
        )),
    );

    builder.mutations.push(
        Field::new(
            "subscriptionSyncOneFeedsFull",
            TypeRef::named_nn(SyncOneSubscriptionInfo::object_type_name()),
            move |ctx| {
                FieldFuture::new(async move {
                    let auth_user_info = ctx.data::<AuthUserInfo>()?;

                    let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;
                    let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

                    let filter_input: SyncOneSubscriptionFilterInput = ctx
                        .args
                        .get(SyncOneSubscriptionFilterInput::arg_name())
                        .unwrap()
                        .deserialize()?;

                    let subscription_model = subscriptions::Model::find_by_id_and_subscriber_id(
                        app_ctx.as_ref(),
                        filter_input.id,
                        subscriber_id,
                    )
                    .await?;

                    let subscription =
                        subscriptions::Subscription::try_from_model(&subscription_model)?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            auth_user_info.subscriber_auth.subscriber_id,
                            SubscriberTask::SyncOneSubscriptionFeedsFull(subscription.into()),
                        )
                        .await?;

                    Ok(Some(FieldValue::owned_any(SyncOneSubscriptionInfo {
                        task_id: task_id.to_string(),
                    })))
                })
            },
        )
        .argument(InputValue::new(
            SyncOneSubscriptionFilterInput::arg_name(),
            TypeRef::named_nn(SyncOneSubscriptionFilterInput::input_type_name()),
        )),
    );

    builder.mutations.push(
        Field::new(
            "subscriptionSyncOneSources",
            TypeRef::named_nn(SyncOneSubscriptionInfo::object_type_name()),
            move |ctx| {
                FieldFuture::new(async move {
                    let auth_user_info = ctx.data::<AuthUserInfo>()?;
                    let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

                    let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

                    let filter_input: SyncOneSubscriptionFilterInput = ctx
                        .args
                        .get(SyncOneSubscriptionFilterInput::arg_name())
                        .unwrap()
                        .deserialize()?;

                    let subscription_model = subscriptions::Model::find_by_id_and_subscriber_id(
                        app_ctx.as_ref(),
                        filter_input.id,
                        subscriber_id,
                    )
                    .await?;

                    let subscription =
                        subscriptions::Subscription::try_from_model(&subscription_model)?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            auth_user_info.subscriber_auth.subscriber_id,
                            SubscriberTask::SyncOneSubscriptionSources(subscription.into()),
                        )
                        .await?;

                    Ok(Some(FieldValue::owned_any(SyncOneSubscriptionInfo {
                        task_id: task_id.to_string(),
                    })))
                })
            },
        )
        .argument(InputValue::new(
            SyncOneSubscriptionFilterInput::arg_name(),
            TypeRef::named_nn(SyncOneSubscriptionFilterInput::input_type_name()),
        )),
    );

    builder
}
