use async_graphql::dynamic::*;
use once_cell::sync::OnceCell;
use sea_orm::{DatabaseConnection, EntityTrait, Iterable};
use seaography::{Builder, BuilderContext, FilterType, FilterTypesMapHelper};

use crate::graphql::{
    infra::{
        filter::{
            JSONB_FILTER_NAME, SUBSCRIBER_ID_FILTER_INFO, init_custom_filter_info,
            register_jsonb_input_filter_to_dynamic_schema, subscriber_id_condition_function,
        },
        guard::{guard_entity_with_subscriber_id, guard_field_with_subscriber_id},
        transformer::{
            build_filter_condition_transformer, build_mutation_input_object_transformer,
        },
        util::{get_entity_column_key, get_entity_key},
    },
    views::register_subscriptions_to_schema,
};

pub static CONTEXT: OnceCell<BuilderContext> = OnceCell::new();

fn restrict_filter_input_for_entity<T>(
    context: &mut BuilderContext,
    column: &T::Column,
    filter_type: Option<FilterType>,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let key = get_entity_column_key::<T>(context, column);
    context.filter_types.overwrites.insert(key, filter_type);
}

fn restrict_jsonb_filter_input_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_column_key = get_entity_column_key::<T>(context, column);
    context.filter_types.overwrites.insert(
        entity_column_key.clone(),
        Some(FilterType::Custom(JSONB_FILTER_NAME.to_string())),
    );
}

fn restrict_subscriber_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    let entity_column_key = get_entity_column_key::<T>(context, column);
    context.guards.entity_guards.insert(
        entity_key.clone(),
        guard_entity_with_subscriber_id::<T>(context, column),
    );
    context.guards.field_guards.insert(
        entity_column_key.clone(),
        guard_field_with_subscriber_id::<T>(context, column),
    );
    context.filter_types.overwrites.insert(
        entity_column_key.clone(),
        Some(FilterType::Custom(
            SUBSCRIBER_ID_FILTER_INFO.get().unwrap().type_name.clone(),
        )),
    );
    context.filter_types.condition_functions.insert(
        entity_column_key.clone(),
        subscriber_id_condition_function::<T>(context, column),
    );
    context.transformers.filter_conditions_transformers.insert(
        entity_key.clone(),
        build_filter_condition_transformer::<T>(context, column),
    );
    context
        .transformers
        .mutation_input_object_transformers
        .insert(
            entity_key,
            build_mutation_input_object_transformer::<T>(context, column),
        );
    context
        .entity_input
        .insert_skips
        .push(entity_column_key.clone());
    context.entity_input.update_skips.push(entity_column_key);
}

pub fn build_schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    use crate::models::*;
    init_custom_filter_info();
    let context = CONTEXT.get_or_init(|| {
        let mut context = BuilderContext::default();

        context.pagination_input.type_name = "PaginationInput".to_string();
        context.pagination_info_object.type_name = "PaginationInfo".to_string();
        context.cursor_input.type_name = "CursorInput".to_string();
        context.offset_input.type_name = "OffsetInput".to_string();
        context.page_input.type_name = "PageInput".to_string();
        context.page_info_object.type_name = "PageInfo".to_string();

        restrict_subscriber_for_entity::<bangumi::Entity>(
            &mut context,
            &bangumi::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<downloaders::Entity>(
            &mut context,
            &downloaders::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<downloads::Entity>(
            &mut context,
            &downloads::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<episodes::Entity>(
            &mut context,
            &episodes::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<subscriptions::Entity>(
            &mut context,
            &subscriptions::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<subscribers::Entity>(
            &mut context,
            &subscribers::Column::Id,
        );
        restrict_subscriber_for_entity::<subscription_bangumi::Entity>(
            &mut context,
            &subscription_bangumi::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<subscription_episode::Entity>(
            &mut context,
            &subscription_episode::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<subscriber_tasks::Entity>(
            &mut context,
            &subscriber_tasks::Column::SubscriberId,
        );
        restrict_subscriber_for_entity::<credential_3rd::Entity>(
            &mut context,
            &credential_3rd::Column::SubscriberId,
        );
        restrict_jsonb_filter_input_for_entity::<subscriber_tasks::Entity>(
            &mut context,
            &subscriber_tasks::Column::Job,
        );
        for column in subscribers::Column::iter() {
            if !matches!(column, subscribers::Column::Id) {
                restrict_filter_input_for_entity::<subscribers::Entity>(
                    &mut context,
                    &column,
                    None,
                );
            }
        }
        context
    });
    let mut builder = Builder::new(context, database.clone());

    {
        let filter_types_map_helper = FilterTypesMapHelper { context };

        builder.schema = builder.schema.register(
            filter_types_map_helper.generate_filter_input(SUBSCRIBER_ID_FILTER_INFO.get().unwrap()),
        );
        builder.schema = register_jsonb_input_filter_to_dynamic_schema(builder.schema);
    }

    {
        builder.register_entity::<subscribers::Entity>(
            <subscribers::RelatedEntity as sea_orm::Iterable>::iter()
                .map(|rel| seaography::RelationBuilder::get_relation(&rel, builder.context))
                .collect(),
        );
        builder = builder.register_entity_dataloader_one_to_one(subscribers::Entity, tokio::spawn);
        builder = builder.register_entity_dataloader_one_to_many(subscribers::Entity, tokio::spawn);
    }

    seaography::register_entities!(
        builder,
        [
            bangumi,
            downloaders,
            downloads,
            episodes,
            subscription_bangumi,
            subscription_episode,
            subscriptions,
            subscriber_tasks,
            credential_3rd
        ]
    );

    {
        builder.register_enumeration::<downloads::DownloadStatus>();
        builder.register_enumeration::<subscriptions::SubscriptionCategory>();
        builder.register_enumeration::<downloaders::DownloaderCategory>();
        builder.register_enumeration::<downloads::DownloadMime>();
        builder.register_enumeration::<credential_3rd::Credential3rdType>();
    }

    {
        builder = register_subscriptions_to_schema(builder);
    }

    let schema = builder.schema_builder();

    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    schema
        .data(database)
        .finish()
        .inspect_err(|e| tracing::error!(e = ?e))
}
