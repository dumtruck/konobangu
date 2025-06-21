use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{graphql::domains::subscribers::restrict_subscriber_for_entity, models::bangumi};

pub fn register_bangumi_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<bangumi::Entity>(context, &bangumi::Column::SubscriberId);
}

pub fn register_bangumi_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<bangumi::BangumiType>();
    seaography::register_entity!(builder, bangumi);

    builder
}
