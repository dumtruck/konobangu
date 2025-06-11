use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::infra::json::restrict_jsonb_filter_input_for_entity, models::subscriber_tasks,
};

pub fn register_subscriber_tasks_to_schema_context(context: &mut BuilderContext) {
    restrict_jsonb_filter_input_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::Job,
    );
}

pub fn register_subscriber_tasks_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.register_enumeration::<subscriber_tasks::SubscriberTaskType>();
    builder
}
