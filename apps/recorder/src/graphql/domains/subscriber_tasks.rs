use std::{ops::Deref, sync::Arc};

use async_graphql::dynamic::{FieldValue, TypeRef};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, prelude::Expr,
    sea_query::Query,
};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, EntityDeleteMutationBuilder, EntityObjectBuilder,
    EntityQueryFieldBuilder, get_filter_conditions,
};

use crate::{
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::generate_entity_filter_mutation_field,
            json::{convert_jsonb_output_case_for_entity, restrict_jsonb_filter_input_for_entity},
        },
    },
    models::subscriber_tasks,
    task::{ApalisJobs, ApalisSchema},
};

pub fn register_subscriber_tasks_entity_mutations(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    let context = builder.context;
    {
        let entitity_delete_mutation_builder = EntityDeleteMutationBuilder { context };
        let delete_mutation = generate_entity_filter_mutation_field::<subscriber_tasks::Entity, _, _>(
            context,
            entitity_delete_mutation_builder.type_name::<subscriber_tasks::Entity>(),
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

                    Ok::<_, RecorderError>(Some(FieldValue::value(result.rows_affected() as i32)))
                })
            }),
        );
        builder.mutations.push(delete_mutation);
    }
    {
        let entity_object_builder = EntityObjectBuilder { context };
        let entity_query_field = EntityQueryFieldBuilder { context };
        let entity_retry_one_mutation_name = format!(
            "{}RetryOne",
            entity_query_field.type_name::<subscriber_tasks::Entity>()
        );
        let retry_one_mutation =
            generate_entity_filter_mutation_field::<subscriber_tasks::Entity, _, _>(
                context,
                entity_retry_one_mutation_name,
                TypeRef::named_nn(entity_object_builder.type_name::<subscriber_tasks::Entity>()),
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
                            .ok_or_else(|| RecorderError::ModelEntityNotFound {
                                entity: "SubscriberTask".into(),
                            })?;

                        let task = app_ctx.task();
                        task.retry_subscriber_task(job_id.clone()).await?;

                        let task_model = subscriber_tasks::Entity::find()
                            .filter(subscriber_tasks::Column::Id.eq(&job_id))
                            .one(db)
                            .await?
                            .ok_or_else(|| RecorderError::ModelEntityNotFound {
                                entity: "SubscriberTask".into(),
                            })?;

                        Ok::<_, RecorderError>(Some(FieldValue::owned_any(task_model)))
                    })
                }),
            );
        builder.mutations.push(retry_one_mutation);
    }

    builder
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
    builder = register_subscriber_tasks_entity_mutations(builder);
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskType>();
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskStatus>();
    builder
}
