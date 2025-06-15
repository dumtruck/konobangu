use std::sync::Arc;

use async_graphql::dynamic::{FieldValue, TypeRef};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seaography::{
    Builder as SeaographyBuilder, EntityObjectBuilder, EntityQueryFieldBuilder,
    get_filter_conditions,
};

use crate::{
    errors::RecorderError,
    graphql::infra::custom::generate_entity_filter_mutation_field,
    models::{
        subscriber_tasks,
        subscriptions::{self, SubscriptionTrait},
    },
    task::SubscriberTask,
};

pub fn register_subscriptions_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    let context = builder.context;

    let entity_object_builder = EntityObjectBuilder { context };
    let entity_query_field = EntityQueryFieldBuilder { context };

    {
        let sync_one_feeds_incremental_mutation_name = format!(
            "{}SyncOneFeedsIncremental",
            entity_query_field.type_name::<subscriptions::Entity>()
        );

        let sync_one_feeds_incremental_mutation = generate_entity_filter_mutation_field::<
            subscriptions::Entity,
            _,
            _,
        >(
            builder.context,
            sync_one_feeds_incremental_mutation_name,
            TypeRef::named_nn(entity_object_builder.type_name::<subscriber_tasks::Entity>()),
            Arc::new(|resolver_ctx, app_ctx, filters| {
                let filters_condition =
                    get_filter_conditions::<subscriptions::Entity>(resolver_ctx, context, filters);

                Box::pin(async move {
                    let db = app_ctx.db();

                    let subscription_model = subscriptions::Entity::find()
                        .filter(filters_condition)
                        .one(db)
                        .await?
                        .ok_or_else(|| RecorderError::ModelEntityNotFound {
                            entity: "Subscription".into(),
                        })?;

                    let subscription =
                        subscriptions::Subscription::try_from_model(&subscription_model)?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            subscription_model.subscriber_id,
                            SubscriberTask::SyncOneSubscriptionFeedsIncremental(
                                subscription.into(),
                            ),
                        )
                        .await?;

                    let task_model = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(task_id.to_string()))
                        .one(db)
                        .await?
                        .ok_or_else(|| RecorderError::ModelEntityNotFound {
                            entity: "SubscriberTask".into(),
                        })?;

                    Ok(Some(FieldValue::owned_any(task_model)))
                })
            }),
        );

        builder.mutations.push(sync_one_feeds_incremental_mutation);
    }
    {
        let sync_one_feeds_full_mutation_name = format!(
            "{}SyncOneFeedsFull",
            entity_query_field.type_name::<subscriptions::Entity>()
        );

        let sync_one_feeds_full_mutation = generate_entity_filter_mutation_field::<
            subscriptions::Entity,
            _,
            _,
        >(
            builder.context,
            sync_one_feeds_full_mutation_name,
            TypeRef::named_nn(entity_object_builder.type_name::<subscriber_tasks::Entity>()),
            Arc::new(|resolver_ctx, app_ctx, filters| {
                let filters_condition =
                    get_filter_conditions::<subscriptions::Entity>(resolver_ctx, context, filters);

                Box::pin(async move {
                    let db = app_ctx.db();

                    let subscription_model = subscriptions::Entity::find()
                        .filter(filters_condition)
                        .one(db)
                        .await?
                        .ok_or_else(|| RecorderError::ModelEntityNotFound {
                            entity: "Subscription".into(),
                        })?;

                    let subscription =
                        subscriptions::Subscription::try_from_model(&subscription_model)?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            subscription_model.subscriber_id,
                            SubscriberTask::SyncOneSubscriptionFeedsFull(subscription.into()),
                        )
                        .await?;

                    let task_model = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(task_id.to_string()))
                        .one(db)
                        .await?
                        .ok_or_else(|| RecorderError::ModelEntityNotFound {
                            entity: "SubscriberTask".into(),
                        })?;

                    Ok(Some(FieldValue::owned_any(task_model)))
                })
            }),
        );

        builder.mutations.push(sync_one_feeds_full_mutation);
    }

    {
        let sync_one_sources_mutation_name = format!(
            "{}SyncOneSources",
            entity_query_field.type_name::<subscriptions::Entity>()
        );

        let sync_one_sources_mutation = generate_entity_filter_mutation_field::<
            subscriptions::Entity,
            _,
            _,
        >(
            builder.context,
            sync_one_sources_mutation_name,
            TypeRef::named_nn(entity_object_builder.type_name::<subscriber_tasks::Entity>()),
            Arc::new(|resolver_ctx, app_ctx, filters| {
                let filters_condition =
                    get_filter_conditions::<subscriptions::Entity>(resolver_ctx, context, filters);

                Box::pin(async move {
                    let db = app_ctx.db();

                    let subscription_model = subscriptions::Entity::find()
                        .filter(filters_condition)
                        .one(db)
                        .await?
                        .ok_or_else(|| RecorderError::ModelEntityNotFound {
                            entity: "Subscription".into(),
                        })?;

                    let subscription =
                        subscriptions::Subscription::try_from_model(&subscription_model)?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            subscription_model.subscriber_id,
                            SubscriberTask::SyncOneSubscriptionSources(subscription.into()),
                        )
                        .await?;

                    let task_model = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(task_id.to_string()))
                        .one(db)
                        .await?
                        .ok_or_else(|| RecorderError::ModelEntityNotFound {
                            entity: "SubscriberTask".into(),
                        })?;

                    Ok(Some(FieldValue::owned_any(task_model)))
                })
            }),
        );

        builder.mutations.push(sync_one_sources_mutation);
    }

    builder
}
