use async_graphql::dynamic::Scalar;
use seaography::{Builder as SeaographyBuilder, BuilderContext, ConvertedType};

use crate::{
    graphql::infra::{
        json::restrict_jsonb_filter_input_for_entity,
        util::{get_column_key, get_entity_key},
    },
    models::subscriber_tasks::{self, SubscriberTask},
};

pub fn register_subscriber_tasks_to_schema_context(context: &mut BuilderContext) {
    let entity_key = get_entity_key::<subscriber_tasks::Entity>(context);
    let column_name =
        get_column_key::<subscriber_tasks::Entity>(context, &subscriber_tasks::Column::Job);
    let column_name = context.entity_object.column_name.as_ref()(&entity_key, &column_name);
    context.types.overwrites.insert(
        column_name,
        ConvertedType::Custom(String::from("SubscriberTask")),
    );
    restrict_jsonb_filter_input_for_entity::<subscriber_tasks::Entity>(
        context,
        &subscriber_tasks::Column::Job,
    );
}

pub fn register_subscriber_tasks_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    let subscriber_tasks_scalar = Scalar::new("SubscriberTasks")
        .description("The subscriber tasks")
        .validator(|value| -> bool {
            if let Ok(json) = value.clone().into_json() {
                serde_json::from_value::<SubscriberTask>(json).is_ok()
            } else {
                false
            }
        });

    builder.schema = builder.schema.register(subscriber_tasks_scalar);
    builder
}
