use async_graphql::dynamic::*;
use once_cell::sync::OnceCell;
use sea_orm::{DatabaseConnection, EntityTrait, Iterable};
use seaography::{Builder, BuilderContext, FilterType, FnGuard};

use super::util::{get_entity_column_key, get_entity_key};
use crate::graphql::guard::guard_entity_with_subscriber_id;

static CONTEXT: OnceCell<BuilderContext> = OnceCell::new();

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

fn restrict_subscriber_for_entity<T>(
    context: &mut BuilderContext,
    column: &T::Column,
    entity_guard: impl FnOnce(&BuilderContext, &T::Column) -> FnGuard,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    context
        .guards
        .entity_guards
        .insert(entity_key, entity_guard(context, column));
}

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    use crate::models::*;
    let context = CONTEXT.get_or_init(|| {
        let mut context = BuilderContext::default();
        restrict_subscriber_for_entity::<bangumi::Entity>(
            &mut context,
            &bangumi::Column::SubscriberId,
            guard_entity_with_subscriber_id::<bangumi::Entity>,
        );
        restrict_subscriber_for_entity::<downloaders::Entity>(
            &mut context,
            &downloaders::Column::SubscriberId,
            guard_entity_with_subscriber_id::<downloaders::Entity>,
        );
        restrict_subscriber_for_entity::<downloads::Entity>(
            &mut context,
            &downloads::Column::SubscriberId,
            guard_entity_with_subscriber_id::<downloads::Entity>,
        );
        restrict_subscriber_for_entity::<episodes::Entity>(
            &mut context,
            &episodes::Column::SubscriberId,
            guard_entity_with_subscriber_id::<episodes::Entity>,
        );
        restrict_subscriber_for_entity::<subscriptions::Entity>(
            &mut context,
            &subscriptions::Column::SubscriberId,
            guard_entity_with_subscriber_id::<subscriptions::Entity>,
        );
        restrict_subscriber_for_entity::<subscribers::Entity>(
            &mut context,
            &subscribers::Column::Id,
            guard_entity_with_subscriber_id::<subscribers::Entity>,
        );
        restrict_subscriber_for_entity::<subscription_bangumi::Entity>(
            &mut context,
            &subscription_bangumi::Column::SubscriberId,
            guard_entity_with_subscriber_id::<subscription_bangumi::Entity>,
        );
        restrict_subscriber_for_entity::<subscription_episode::Entity>(
            &mut context,
            &subscription_episode::Column::SubscriberId,
            guard_entity_with_subscriber_id::<subscription_episode::Entity>,
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
            subscriptions
        ]
    );

    {
        builder.register_enumeration::<downloads::DownloadStatus>();
        builder.register_enumeration::<subscriptions::SubscriptionCategory>();
        builder.register_enumeration::<downloaders::DownloaderCategory>();
        builder.register_enumeration::<downloads::DownloadMime>();
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
        // .extension(GraphqlAuthExtension)
        .finish()
        .inspect_err(|e| tracing::error!(e = ?e))
}
