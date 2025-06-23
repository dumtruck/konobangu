use std::sync::Arc;

use async_graphql::dynamic::ResolverContext;
use sea_orm::Value as SeaValue;
use seaography::{Builder as SeaographyBuilder, BuilderContext, SeaResult};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::util::{get_entity_column_key, get_entity_key},
    },
    models::feeds,
};

pub fn register_feeds_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<feeds::Entity>(context, &feeds::Column::SubscriberId);
    {
        let entity_column_key =
            get_entity_column_key::<feeds::Entity>(context, &feeds::Column::Token);
        let entity_key = get_entity_key::<feeds::Entity>(context);
        let entity_name = context.entity_query_field.type_name.as_ref()(&entity_key);
        let entity_create_one_mutation_field_name = Arc::new(format!(
            "{}{}",
            entity_name, context.entity_create_one_mutation.mutation_suffix
        ));
        let entity_create_batch_mutation_field_name = Arc::new(format!(
            "{}{}",
            entity_name,
            context.entity_create_batch_mutation.mutation_suffix.clone()
        ));

        context.types.input_none_conversions.insert(
            entity_column_key,
            Box::new(
                move |context: &ResolverContext| -> SeaResult<Option<SeaValue>> {
                    let field_name = context.field().name();
                    if field_name == entity_create_one_mutation_field_name.as_str()
                        || field_name == entity_create_batch_mutation_field_name.as_str()
                    {
                        Ok(Some(SeaValue::String(Some(Box::new(nanoid::nanoid!())))))
                    } else {
                        Ok(None)
                    }
                },
            ),
        );
    }
}

pub fn register_feeds_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<feeds::FeedType>();
    builder.register_enumeration::<feeds::FeedSource>();
    seaography::register_entity!(builder, feeds);

    builder
}
