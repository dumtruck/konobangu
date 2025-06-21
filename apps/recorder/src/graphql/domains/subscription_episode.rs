use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::domains::subscribers::restrict_subscriber_for_entity, models::subscription_episode,
};

pub fn register_subscription_episode_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<subscription_episode::Entity>(
        context,
        &subscription_episode::Column::SubscriberId,
    );
}

pub fn register_subscription_episode_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    seaography::register_entity!(builder, subscription_episode);

    builder
}
