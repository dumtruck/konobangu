use std::sync::Arc;

use async_graphql::dynamic::{ResolverContext, ValueAccessor};
use sea_orm::EntityTrait;
use seaography::{BuilderContext, FnGuard, GuardAction};

use super::util::get_entity_key;
use crate::{
    auth::{AuthError, AuthUserInfo},
    graphql::util::get_column_key,
};

fn guard_data_object_accessor_with_subscriber_id(
    value: ValueAccessor<'_>,
    column_name: &str,
    subscriber_id: i32,
) -> async_graphql::Result<()> {
    let obj = value.object()?;

    let subscriber_id_value = obj.try_get(column_name)?;

    let id = subscriber_id_value.i64()?;

    if id == subscriber_id as i64 {
        Ok(())
    } else {
        Err(async_graphql::Error::new("subscriber permission denied"))
    }
}

fn guard_data_object_accessor_with_optional_subscriber_id(
    value: ValueAccessor<'_>,
    column_name: &str,
    subscriber_id: i32,
) -> async_graphql::Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let obj = value.object()?;

    if let Some(subscriber_id_value) = obj.get(column_name) {
        let id = subscriber_id_value.i64()?;
        if id == subscriber_id as i64 {
            Ok(())
        } else {
            Err(async_graphql::Error::new("subscriber permission denied"))
        }
    } else {
        Ok(())
    }
}

fn guard_filter_object_accessor_with_subscriber_id(
    value: ValueAccessor<'_>,
    column_name: &str,
    subscriber_id: i32,
) -> async_graphql::Result<()> {
    let obj = value.object()?;
    let subscriber_id_filter_input_value = obj.try_get(column_name)?;

    let subscriber_id_filter_input_obj = subscriber_id_filter_input_value.object()?;

    let subscriber_id_value = subscriber_id_filter_input_obj.try_get("eq")?;

    let id = subscriber_id_value.i64()?;
    if id == subscriber_id as i64 {
        Ok(())
    } else {
        Err(async_graphql::Error::new("subscriber permission denied"))
    }
}

pub fn guard_entity_with_subscriber_id<T>(context: &BuilderContext, column: &T::Column) -> FnGuard
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    let entity_name = context.entity_query_field.type_name.as_ref()(&entity_key);
    let column_key = get_column_key::<T>(context, column);
    let column_name = Arc::new(context.entity_object.column_name.as_ref()(
        &entity_key,
        &column_key,
    ));
    let entity_create_one_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name, context.entity_create_one_mutation.mutation_suffix
    ));
    let entity_create_one_mutation_data_field_name =
        Arc::new(context.entity_create_one_mutation.data_field.clone());
    let entity_create_batch_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name,
        context.entity_create_batch_mutation.mutation_suffix.clone()
    ));
    let entity_create_batch_mutation_data_field_name =
        Arc::new(context.entity_create_batch_mutation.data_field.clone());
    let entity_delete_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name,
        context.entity_delete_mutation.mutation_suffix.clone()
    ));
    let entity_delete_filter_field_name =
        Arc::new(context.entity_delete_mutation.filter_field.clone());
    let entity_update_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name, context.entity_update_mutation.mutation_suffix
    ));
    let entity_update_mutation_filter_field_name =
        Arc::new(context.entity_update_mutation.filter_field.clone());
    let entity_update_mutation_data_field_name =
        Arc::new(context.entity_update_mutation.data_field.clone());
    let entity_query_field_name = Arc::new(entity_name);
    let entity_query_filter_field_name = Arc::new(context.entity_query_field.filters.clone());
    Box::new(move |context: &ResolverContext| -> GuardAction {
        match context.ctx.data::<AuthUserInfo>() {
            Ok(user_info) => {
                let subscriber_id = user_info.subscriber_auth.subscriber_id;
                let validation_result = match context.field().name() {
                    field if field == entity_create_one_mutation_field_name.as_str() => context
                        .args
                        .try_get(&entity_create_one_mutation_data_field_name)
                        .and_then(|data_value| {
                            guard_data_object_accessor_with_subscriber_id(
                                data_value,
                                &column_name,
                                subscriber_id,
                            )
                        }),
                    field if field == entity_create_batch_mutation_field_name.as_str() => context
                        .args
                        .try_get(&entity_create_batch_mutation_data_field_name)
                        .and_then(|data_value| {
                            data_value.list().and_then(|data_list| {
                                data_list.iter().try_for_each(|data_item_value| {
                                    guard_data_object_accessor_with_subscriber_id(
                                        data_item_value,
                                        &column_name,
                                        subscriber_id,
                                    )
                                })
                            })
                        }),
                    field if field == entity_delete_mutation_field_name.as_str() => context
                        .args
                        .try_get(&entity_delete_filter_field_name)
                        .and_then(|filter_value| {
                            guard_filter_object_accessor_with_subscriber_id(
                                filter_value,
                                &column_name,
                                subscriber_id,
                            )
                        }),
                    field if field == entity_update_mutation_field_name.as_str() => context
                        .args
                        .try_get(&entity_update_mutation_filter_field_name)
                        .and_then(|filter_value| {
                            guard_filter_object_accessor_with_subscriber_id(
                                filter_value,
                                &column_name,
                                subscriber_id,
                            )
                        })
                        .and_then(|_| {
                            match context.args.get(&entity_update_mutation_data_field_name) {
                                Some(data_value) => {
                                    guard_data_object_accessor_with_optional_subscriber_id(
                                        data_value,
                                        &column_name,
                                        subscriber_id,
                                    )
                                }
                                None => Ok(()),
                            }
                        }),
                    field if field == entity_query_field_name.as_str() => context
                        .args
                        .try_get(&entity_query_filter_field_name)
                        .and_then(|filter_value| {
                            guard_filter_object_accessor_with_subscriber_id(
                                filter_value,
                                &column_name,
                                subscriber_id,
                            )
                        }),
                    field => Err(async_graphql::Error::new(format!(
                        "unsupport graphql field {}",
                        field
                    ))),
                };
                match validation_result.map_err(AuthError::GraphQLPermissionError) {
                    Ok(_) => GuardAction::Allow,
                    Err(err) => GuardAction::Block(Some(err.to_string())),
                }
            }
            Err(err) => GuardAction::Block(Some(err.message)),
        }
    })
}
