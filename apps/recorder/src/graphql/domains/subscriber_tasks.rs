use std::{ops::Deref, sync::Arc};

use async_graphql::dynamic::{FieldValue, TypeRef, ValueAccessor};
use convert_case::Case;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, Iterable, QueryFilter, QuerySelect, QueryTrait,
    prelude::Expr, sea_query::Query,
};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, GuardAction, get_filter_conditions,
};

use crate::{
    auth::AuthUserInfo,
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::{
                generate_entity_create_one_mutation_field,
                generate_entity_default_insert_input_object,
                generate_entity_filtered_mutation_field,
            },
            json::{
                convert_jsonb_output_case_for_entity, restrict_jsonb_filter_input_for_entity,
                validate_jsonb_input_for_entity,
            },
            name::{
                get_column_name, get_entity_and_column_name, get_entity_basic_type_name,
                get_entity_create_batch_mutation_data_field_name,
                get_entity_create_batch_mutation_field_name,
                get_entity_create_one_mutation_data_field_name,
                get_entity_create_one_mutation_field_name, get_entity_custom_mutation_field_name,
                get_entity_delete_mutation_field_name, get_entity_update_mutation_field_name,
            },
        },
    },
    models::subscriber_tasks,
    task::{ApalisJobs, ApalisSchema},
};

pub fn check_entity_and_task_subscriber_id_matches(
    value_accessor: &ValueAccessor<'_>,
    subscriber_id: i32,
    subscriber_id_column_name: &str,
    subscriber_task_column_name: &str,
) -> bool {
    value_accessor.object().is_ok_and(|input_object| {
        input_object
            .get(subscriber_task_column_name)
            .and_then(|subscriber_task_value| subscriber_task_value.object().ok())
            .and_then(|subscriber_task_object| {
                subscriber_task_object
                    .get("subscriber_id")
                    .and_then(|job_subscriber_id| job_subscriber_id.i64().ok())
            })
            .is_some_and(|subscriber_task_subscriber_id| {
                subscriber_task_subscriber_id as i32
                    == input_object
                        .get(subscriber_id_column_name)
                        .and_then(|subscriber_id_object| subscriber_id_object.i64().ok())
                        .map(|subscriber_id| subscriber_id as i32)
                        .unwrap_or(subscriber_id)
            })
    })
}

fn skip_columns_for_entity_input(context: &mut BuilderContext) {
    for column in subscriber_tasks::Column::iter() {
        if matches!(
            column,
            subscriber_tasks::Column::Job
                | subscriber_tasks::Column::Id
                | subscriber_tasks::Column::SubscriberId
                | subscriber_tasks::Column::Priority
                | subscriber_tasks::Column::MaxAttempts
        ) {
            continue;
        }
        let entity_column_key =
            get_entity_and_column_name::<subscriber_tasks::Entity>(context, &column);
        context.entity_input.insert_skips.push(entity_column_key);
    }
}

pub fn register_subscriber_tasks_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::SubscriberId,
    );
    restrict_jsonb_filter_input_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::Job,
    );
    convert_jsonb_output_case_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::Job,
        Case::Camel,
    );
    validate_jsonb_input_for_entity::<subscriber_tasks::Entity, subscriber_tasks::SubscriberTask>(
        context,
        &subscriber_tasks::Column::Job,
    );
    skip_columns_for_entity_input(context);

    context.guards.field_guards.insert(
        get_entity_and_column_name::<subscriber_tasks::Entity>(
            context,
            &subscriber_tasks::Column::Job,
        ),
        {
            let create_one_mutation_field_name =
                Arc::new(get_entity_create_one_mutation_field_name::<
                    subscriber_tasks::Entity,
                >(context));
            let create_one_mutation_data_field_name =
                Arc::new(get_entity_create_one_mutation_data_field_name(context).to_string());
            let create_batch_mutation_field_name =
                Arc::new(get_entity_create_batch_mutation_field_name::<
                    subscriber_tasks::Entity,
                >(context));
            let create_batch_mutation_data_field_name =
                Arc::new(get_entity_create_batch_mutation_data_field_name(context).to_string());
            let update_mutation_field_name = Arc::new(get_entity_update_mutation_field_name::<
                subscriber_tasks::Entity,
            >(context));
            let job_column_name = Arc::new(get_column_name::<subscriber_tasks::Entity>(
                context,
                &subscriber_tasks::Column::Job,
            ));
            let subscriber_id_column_name = Arc::new(get_column_name::<subscriber_tasks::Entity>(
                context,
                &subscriber_tasks::Column::SubscriberId,
            ));

            Box::new(move |resolve_context| {
                let field_name = resolve_context.field().name();
                let subscriber_id = resolve_context
                    .data_opt::<AuthUserInfo>()
                    .unwrap()
                    .subscriber_auth
                    .subscriber_id;
                let matched_subscriber_id = match field_name {
                    field if field == create_one_mutation_field_name.as_str() => resolve_context
                        .args
                        .get(create_one_mutation_data_field_name.as_str())
                        .is_some_and(|value_accessor| {
                            check_entity_and_task_subscriber_id_matches(
                                &value_accessor,
                                subscriber_id,
                                subscriber_id_column_name.as_str(),
                                job_column_name.as_str(),
                            )
                        }),
                    field if field == create_batch_mutation_field_name.as_str() => resolve_context
                        .args
                        .get(create_batch_mutation_data_field_name.as_str())
                        .and_then(|value| value.list().ok())
                        .is_some_and(|list| {
                            list.iter().all(|value| {
                                check_entity_and_task_subscriber_id_matches(
                                    &value,
                                    subscriber_id,
                                    subscriber_id_column_name.as_str(),
                                    job_column_name.as_str(),
                                )
                            })
                        }),
                    field if field == update_mutation_field_name.as_str() => {
                        unreachable!("subscriberTask entity do not support update job")
                    }
                    _ => true,
                };
                if matched_subscriber_id {
                    GuardAction::Allow
                } else {
                    GuardAction::Block(Some(
                        "subscriber_id mismatch between entity and job".to_string(),
                    ))
                }
            })
        },
    );
}

pub fn register_subscriber_tasks_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.register_entity::<subscriber_tasks::Entity>(
        <subscriber_tasks::RelatedEntity as sea_orm::Iterable>::iter()
            .map(|rel| seaography::RelationBuilder::get_relation(&rel, builder.context))
            .collect(),
    );
    builder = builder.register_entity_dataloader_one_to_one(subscriber_tasks::Entity, tokio::spawn);
    builder =
        builder.register_entity_dataloader_one_to_many(subscriber_tasks::Entity, tokio::spawn);
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskType>();
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskStatus>();

    let context = builder.context;
    {
        let delete_mutation =
            generate_entity_filtered_mutation_field::<subscriber_tasks::Entity, _, _>(
                context,
                get_entity_delete_mutation_field_name::<subscriber_tasks::Entity>(context),
                TypeRef::named_nn(TypeRef::INT),
                Arc::new(|resolver_ctx, app_ctx, filters| {
                    let filters_condition = get_filter_conditions::<subscriber_tasks::Entity>(
                        resolver_ctx,
                        context,
                        filters,
                    );
                    Box::pin(async move {
                        let db = app_ctx.db();

                        let select_subquery = subscriber_tasks::Entity::find()
                            .select_only()
                            .column(subscriber_tasks::Column::Id)
                            .filter(filters_condition);

                        let delete_query = Query::delete()
                            .from_table((ApalisSchema::Schema, ApalisJobs::Table))
                            .and_where(
                                Expr::col(ApalisJobs::Id).in_subquery(select_subquery.into_query()),
                            )
                            .to_owned();

                        let db_backend = db.deref().get_database_backend();
                        let delete_statement = db_backend.build(&delete_query);

                        let result = db.execute(delete_statement).await?;

                        Ok::<_, RecorderError>(Some(FieldValue::value(
                            result.rows_affected() as i32
                        )))
                    })
                }),
            );
        builder.mutations.push(delete_mutation);
    }
    {
        let entity_retry_one_mutation_name =
            get_entity_custom_mutation_field_name::<subscriber_tasks::Entity>(context, "RetryOne");
        let retry_one_mutation =
            generate_entity_filtered_mutation_field::<subscriber_tasks::Entity, _, _>(
                context,
                entity_retry_one_mutation_name,
                TypeRef::named_nn(get_entity_basic_type_name::<subscriber_tasks::Entity>(
                    context,
                )),
                Arc::new(|resolver_ctx, app_ctx, filters| {
                    let filters_condition = get_filter_conditions::<subscriber_tasks::Entity>(
                        resolver_ctx,
                        context,
                        filters,
                    );
                    Box::pin(async move {
                        let db = app_ctx.db();

                        let job_id = subscriber_tasks::Entity::find()
                            .filter(filters_condition)
                            .select_only()
                            .column(subscriber_tasks::Column::Id)
                            .into_tuple::<String>()
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                            })?;

                        let task = app_ctx.task();
                        task.retry_subscriber_task(job_id.clone()).await?;

                        let task_model = subscriber_tasks::Entity::find()
                            .filter(subscriber_tasks::Column::Id.eq(&job_id))
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                            })?;

                        Ok::<_, RecorderError>(Some(FieldValue::owned_any(task_model)))
                    })
                }),
            );
        builder.mutations.push(retry_one_mutation);
    }
    {
        builder
            .inputs
            .push(generate_entity_default_insert_input_object::<
                subscriber_tasks::Entity,
            >(context));
        let create_one_mutation =
            generate_entity_create_one_mutation_field::<subscriber_tasks::Entity, TypeRef>(
                context,
                None,
                Arc::new(|_resolver_ctx, app_ctx, input_object| {
                    let job_column_name = get_column_name::<subscriber_tasks::Entity>(
                        context,
                        &subscriber_tasks::Column::Job,
                    );
                    let task = input_object
                        .get(job_column_name.as_str())
                        .unwrap()
                        .deserialize::<subscriber_tasks::SubscriberTask>()
                        .unwrap();

                    Box::pin(async move {
                        let task_service = app_ctx.task();

                        let task_id = task_service.add_subscriber_task(task).await?.to_string();

                        let db = app_ctx.db();

                        let task = subscriber_tasks::Entity::find()
                            .filter(subscriber_tasks::Column::Id.eq(&task_id))
                            .one(db)
                            .await?
                            .ok_or_else(|| {
                                RecorderError::from_entity_not_found::<subscriber_tasks::Entity>()
                            })?;

                        Ok::<_, RecorderError>(task)
                    })
                }),
            );
        builder.mutations.push(create_one_mutation);
    }
    builder
}
