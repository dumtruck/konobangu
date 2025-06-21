use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::domains::subscribers::restrict_subscriber_for_entity, models::subscription_bangumi,
};

pub fn register_subscription_bangumi_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscription_bangumi::Entity>(
        context,
        &subscription_bangumi::Column::SubscriberId,
    );
}

pub fn register_subscription_bangumi_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    seaography::register_entity!(builder, subscription_bangumi);

    builder
}
