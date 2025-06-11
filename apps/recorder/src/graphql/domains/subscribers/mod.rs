use sea_orm::{EntityTrait, Iterable};
use seaography::{Builder as SeaographyBuilder, BuilderContext, FilterType, FilterTypesMapHelper};

mod filter;
mod guard;
mod transformer;

use filter::{SUBSCRIBER_ID_FILTER_INFO, generate_subscriber_id_condition_function};
use guard::{guard_entity_with_subscriber_id, guard_field_with_subscriber_id};
use transformer::{
    generate_subscriber_id_filter_condition_transformer,
    generate_subscriber_id_mutation_input_object_transformer,
};

use crate::{
    graphql::infra::util::{get_entity_column_key, get_entity_key},
    models::subscribers,
};

pub fn restrict_subscriber_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
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
            SUBSCRIBER_ID_FILTER_INFO.type_name.clone(),
        )),
    );
    context.filter_types.condition_functions.insert(
        entity_column_key.clone(),
        generate_subscriber_id_condition_function::<T>(context, column),
    );
    context.transformers.filter_conditions_transformers.insert(
        entity_key.clone(),
        generate_subscriber_id_filter_condition_transformer::<T>(context, column),
    );
    context
        .transformers
        .mutation_input_object_transformers
        .insert(
            entity_key,
            generate_subscriber_id_mutation_input_object_transformer::<T>(context, column),
        );
    context
        .entity_input
        .insert_skips
        .push(entity_column_key.clone());
    context.entity_input.update_skips.push(entity_column_key);
}

pub fn register_subscribers_to_schema_context(context: &mut BuilderContext) {
    for column in subscribers::Column::iter() {
        if !matches!(column, subscribers::Column::Id) {
            let key = get_entity_column_key::<subscribers::Entity>(context, &column);
            context.filter_types.overwrites.insert(key, None);
        }
    }
}

pub fn register_subscribers_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    {
        let filter_types_map_helper = FilterTypesMapHelper {
            context: builder.context,
        };

        builder.schema = builder
            .schema
            .register(filter_types_map_helper.generate_filter_input(&SUBSCRIBER_ID_FILTER_INFO));
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

    builder
}
