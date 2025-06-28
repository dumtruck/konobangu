use convert_case::Case;
use sea_orm::Iterable;
use seaography::{Builder as SeaographyBuilder, BuilderContext};

use crate::{
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::{
            custom::register_entity_default_writable,
            json::{
                convert_jsonb_output_case_for_entity, restrict_jsonb_filter_input_for_entity,
                validate_jsonb_input_for_entity,
            },
            name::get_entity_and_column_name,
        },
    },
    models::{cron, subscriber_tasks},
};

fn skip_columns_for_entity_input(context: &mut BuilderContext) {
    for column in cron::Column::iter() {
        if matches!(
            column,
            cron::Column::SubscriberTask
                | cron::Column::Id
                | cron::Column::CronExpr
                | cron::Column::Enabled
                | cron::Column::TimeoutMs
                | cron::Column::MaxAttempts
        ) {
            continue;
        }
        let entity_column_key = get_entity_and_column_name::<cron::Entity>(context, &column);
        context.entity_input.insert_skips.push(entity_column_key);
    }
    for column in cron::Column::iter() {
        if matches!(column, |cron::Column::CronExpr| cron::Column::Enabled
            | cron::Column::TimeoutMs
            | cron::Column::Priority
            | cron::Column::MaxAttempts)
        {
            continue;
        }
        let entity_column_key = get_entity_and_column_name::<cron::Entity>(context, &column);
        context.entity_input.update_skips.push(entity_column_key);
    }
}

pub fn register_cron_to_schema_context(context: &mut BuilderContext) {
    restrict_subscriber_for_entity::<cron::Entity>(context, &cron::Column::SubscriberId);

    restrict_jsonb_filter_input_for_entity::<cron::Entity>(context, &cron::Column::SubscriberTask);
    convert_jsonb_output_case_for_entity::<cron::Entity>(
        context,
        &cron::Column::SubscriberTask,
        Case::Camel,
    );
    validate_jsonb_input_for_entity::<cron::Entity, Option<subscriber_tasks::SubscriberTask>>(
        context,
        &cron::Column::SubscriberTask,
    );
    skip_columns_for_entity_input(context);
}

pub fn register_cron_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    builder.register_enumeration::<cron::CronStatus>();

    builder = register_entity_default_writable!(builder, cron, true);

    builder
}
