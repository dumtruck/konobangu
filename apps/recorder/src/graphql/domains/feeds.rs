use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{graphql::domains::subscribers::restrict_subscriber_for_entity, models::feeds};

pub fn register_feeds_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<feeds::Entity>(context, &feeds::Column::SubscriberId);
}

pub fn register_feeds_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<feeds::FeedType>();
    builder.register_enumeration::<feeds::FeedSource>();
    seaography::register_entity!(builder, feeds);

    builder
}
