use std::sync::Arc;

use async_graphql::dynamic::ResolverContext;
use sea_orm::Value as SeaValue;
use seaography::{Builder as SeaographyBuilder, BuilderContext, SeaResult};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::register_entity_default_writable,
            name::{
                get_entity_and_column_name, get_entity_create_batch_mutation_field_name,
                get_entity_create_one_mutation_field_name,
            },
        },
    },
    models::feeds,
};

pub fn register_feeds_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<feeds::Entity>(context, &feeds::Column::SubscriberId);
    {
        let entity_create_one_mutation_field_name = Arc::new(
            get_entity_create_one_mutation_field_name::<feeds::Entity>(context),
        );
        let entity_create_batch_mutation_field_name =
            Arc::new(get_entity_create_batch_mutation_field_name::<feeds::Entity>(context));

        context.types.input_none_conversions.insert(
            get_entity_and_column_name::<feeds::Entity>(context, &feeds::Column::Token),
            Arc::new(
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

    builder = register_entity_default_writable!(builder, feeds, false);

    builder
}
