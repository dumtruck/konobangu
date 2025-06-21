use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{graphql::domains::subscribers::restrict_subscriber_for_entity, models::downloaders};

pub fn register_downloaders_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<downloaders::Entity>(
        context,
        &downloaders::Column::SubscriberId,
    );
}

pub fn register_downloaders_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<downloaders::DownloaderCategory>();
    seaography::register_entity!(builder, downloaders);

    builder
}
