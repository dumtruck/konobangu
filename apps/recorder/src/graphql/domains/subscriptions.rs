use std::sync::Arc;

use async_graphql::dynamic::{FieldValue, TypeRef};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seaography::{Builder as SeaographyBuilder, BuilderContext, get_filter_conditions};

use crate::{
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::generate_entity_filtered_mutation_field,
            name::{get_entity_basic_type_name, get_entity_custom_mutation_field_name},
        },
    },
    models::{subscriber_tasks, subscriptions},
    task::{
        SyncOneSubscriptionFeedsFullTask, SyncOneSubscriptionFeedsIncrementalTask,
        SyncOneSubscriptionSourcesTask,
    },
};

pub fn register_subscriptions_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscriptions::Entity>(
        context,
        &subscriptions::Column::SubscriberId,
    );
}

pub fn register_subscriptions_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.register_enumeration::<subscriptions::SubscriptionCategory>();
    seaography::register_entity!(builder, subscriptions);

    let context = builder.context;

    {
        let sync_one_feeds_incremental_mutation_name = get_entity_custom_mutation_field_name::<
            subscriptions::Entity,
        >(context, "SyncOneFeedsIncremental");

        let sync_one_feeds_incremental_mutation = generate_entity_filtered_mutation_field::<
            subscriptions::Entity,
            _,
            _,
        >(
            builder.context,
            sync_one_feeds_incremental_mutation_name,
            TypeRef::named_nn(get_entity_basic_type_name::<subscriber_tasks::Entity>(
                context,
            )),
            Arc::new(|resolver_ctx, app_ctx, filters| {
                let filters_condition =
                    get_filter_conditions::<subscriptions::Entity>(resolver_ctx, context, filters);

                Box::pin(async move {
                    let db = app_ctx.db();

                    let subscription_model = subscriptions::Entity::find()
                        .filter(filters_condition)
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriptions::Entity>()
                        })?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            SyncOneSubscriptionFeedsIncrementalTask::builder()
                                .subscriber_id(subscription_model.subscriber_id)
                                .subscription_id(subscription_model.id)
                                .build()
                                .into(),
                        )
                        .await?;

                    let task_model = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(task_id.to_string()))
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                        })?;

                    Ok(Some(FieldValue::owned_any(task_model)))
                })
            }),
        );

        builder.mutations.push(sync_one_feeds_incremental_mutation);
    }
    {
        let sync_one_feeds_full_mutation_name = get_entity_custom_mutation_field_name::<
            subscriptions::Entity,
        >(builder.context, "SyncOneFeedsFull");
        let sync_one_feeds_full_mutation = generate_entity_filtered_mutation_field::<
            subscriptions::Entity,
            _,
            _,
        >(
            builder.context,
            sync_one_feeds_full_mutation_name,
            TypeRef::named_nn(get_entity_basic_type_name::<subscriber_tasks::Entity>(
                context,
            )),
            Arc::new(|resolver_ctx, app_ctx, filters| {
                let filters_condition =
                    get_filter_conditions::<subscriptions::Entity>(resolver_ctx, context, filters);

                Box::pin(async move {
                    let db = app_ctx.db();

                    let subscription_model = subscriptions::Entity::find()
                        .filter(filters_condition)
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriptions::Entity>()
                        })?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            SyncOneSubscriptionFeedsFullTask::builder()
                                .subscriber_id(subscription_model.subscriber_id)
                                .subscription_id(subscription_model.id)
                                .build()
                                .into(),
                        )
                        .await?;

                    let task_model = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(task_id.to_string()))
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                        })?;

                    Ok(Some(FieldValue::owned_any(task_model)))
                })
            }),
        );

        builder.mutations.push(sync_one_feeds_full_mutation);
    }

    {
        let sync_one_sources_mutation_name = get_entity_custom_mutation_field_name::<
            subscriptions::Entity,
        >(context, "SyncOneSources");

        let sync_one_sources_mutation = generate_entity_filtered_mutation_field::<
            subscriptions::Entity,
            _,
            _,
        >(
            builder.context,
            sync_one_sources_mutation_name,
            TypeRef::named_nn(get_entity_basic_type_name::<subscriber_tasks::Entity>(
                context,
            )),
            Arc::new(|resolver_ctx, app_ctx, filters| {
                let filters_condition =
                    get_filter_conditions::<subscriptions::Entity>(resolver_ctx, context, filters);

                Box::pin(async move {
                    let db = app_ctx.db();

                    let subscription_model = subscriptions::Entity::find()
                        .filter(filters_condition)
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriptions::Entity>()
                        })?;

                    let task_service = app_ctx.task();

                    let task_id = task_service
                        .add_subscriber_task(
                            SyncOneSubscriptionSourcesTask::builder()
                                .subscriber_id(subscription_model.subscriber_id)
                                .subscription_id(subscription_model.id)
                                .build()
                                .into(),
                        )
                        .await?;

                    let task_model = subscriber_tasks::Entity::find()
                        .filter(subscriber_tasks::Column::Id.eq(task_id.to_string()))
                        .one(db)
                        .await?
                        .ok_or_else(|| {
                            RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                        })?;

                    Ok(Some(FieldValue::owned_any(task_model)))
                })
            }),
        );

        builder.mutations.push(sync_one_sources_mutation);
    }

    builder
}
