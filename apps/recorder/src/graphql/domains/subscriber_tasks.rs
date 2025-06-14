use std::{ops::Deref, pin::Pin, sync::Arc};

use async_graphql::dynamic::{ResolverContext, ValueAccessor};
use sea_orm::{
    ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, prelude::Expr,
    sea_query::Query,
};
use seaography::{Builder as SeaographyBuilder, BuilderContext, get_filter_conditions};

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::generate_custom_entity_delete_mutation_field,
            json::{convert_jsonb_output_case_for_entity, restrict_jsonb_filter_input_for_entity},
        },
    },
    models::subscriber_tasks,
    task::ApalisJob,
};

pub fn register_subscriber_tasks_entity_mutations(builder: &mut SeaographyBuilder) {
    let context = builder.context;
    let delete_mutation = generate_custom_entity_delete_mutation_field::<subscriber_tasks::Entity>(
        context,
        Arc::new(
            |resolver_ctx: &ResolverContext<'_>,
             app_ctx: Arc<dyn AppContextTrait>,
             filters: Option<ValueAccessor<'_>>|
             -> Pin<Box<dyn Future<Output = RecorderResult<Option<i32>>> + Send>> {
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
                        .from_table(ApalisJob::Table)
                        .and_where(
                            Expr::col(ApalisJob::Id).in_subquery(select_subquery.into_query()),
                        )
                        .to_owned();

                    let db_backend = db.deref().get_database_backend();
                    let delete_statement = db_backend.build(&delete_query);
                    let result = db.execute(delete_statement).await?;

                    Ok::<Option<i32>, RecorderError>(Some(result.rows_affected() as i32))
                })
                    as Pin<Box<dyn Future<Output = RecorderResult<Option<i32>>> + Send>>
            },
        ),
    );
    builder.mutations.push(delete_mutation);
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
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskType>();
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskStatus>();
    builder
}
