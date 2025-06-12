use std::sync::Arc;

use async_graphql::dynamic::{ObjectAccessor, ResolverContext, TypeRef, ValueAccessor};
use lazy_static::lazy_static;
use maplit::btreeset;
use sea_orm::{ColumnTrait, Condition, EntityTrait, Iterable, Value as SeaValue};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, FilterInfo,
    FilterOperation as SeaographqlFilterOperation, FilterType, FilterTypesMapHelper,
    FnFilterCondition, FnGuard, FnInputTypeNoneConversion, GuardAction, SeaResult, SeaographyError,
};

use crate::{
    auth::{AuthError, AuthUserInfo},
    graphql::infra::util::{get_column_key, get_entity_column_key, get_entity_key},
    models::subscribers,
};

lazy_static! {
    pub static ref SUBSCRIBER_ID_FILTER_INFO: FilterInfo = FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    };
}

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
        Err(async_graphql::Error::new("subscriber not match"))
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
            Err(async_graphql::Error::new("subscriber not match"))
        }
    } else {
        Ok(())
    }
}

pub fn guard_entity_with_subscriber_id<T>(_context: &BuilderContext, _column: &T::Column) -> FnGuard
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    Box::new(move |context: &ResolverContext| -> GuardAction {
        match context.ctx.data::<AuthUserInfo>() {
            Ok(_) => GuardAction::Allow,
            Err(err) => GuardAction::Block(Some(err.message)),
        }
    })
}

pub fn guard_field_with_subscriber_id<T>(context: &BuilderContext, column: &T::Column) -> FnGuard
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
    let entity_update_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name, context.entity_update_mutation.mutation_suffix
    ));
    let entity_update_mutation_data_field_name =
        Arc::new(context.entity_update_mutation.data_field.clone());

    Box::new(move |context: &ResolverContext| -> GuardAction {
        match context.ctx.data::<AuthUserInfo>() {
            Ok(user_info) => {
                let subscriber_id = user_info.subscriber_auth.subscriber_id;
                let validation_result = match context.field().name() {
                    field if field == entity_create_one_mutation_field_name.as_str() => {
                        if let Some(data_value) = context
                            .args
                            .get(&entity_create_one_mutation_data_field_name)
                        {
                            guard_data_object_accessor_with_subscriber_id(
                                data_value,
                                &column_name,
                                subscriber_id,
                            )
                            .map_err(|inner_error| {
                                AuthError::from_graphql_dynamic_subscribe_id_guard(
                                    inner_error,
                                    context,
                                    &entity_create_one_mutation_data_field_name,
                                    &column_name,
                                )
                            })
                        } else {
                            Ok(())
                        }
                    }
                    field if field == entity_create_batch_mutation_field_name.as_str() => {
                        if let Some(data_value) = context
                            .args
                            .get(&entity_create_batch_mutation_data_field_name)
                        {
                            data_value
                                .list()
                                .and_then(|data_list| {
                                    data_list.iter().try_for_each(|data_item_value| {
                                        guard_data_object_accessor_with_optional_subscriber_id(
                                            data_item_value,
                                            &column_name,
                                            subscriber_id,
                                        )
                                    })
                                })
                                .map_err(|inner_error| {
                                    AuthError::from_graphql_dynamic_subscribe_id_guard(
                                        inner_error,
                                        context,
                                        &entity_create_batch_mutation_data_field_name,
                                        &column_name,
                                    )
                                })
                        } else {
                            Ok(())
                        }
                    }
                    field if field == entity_update_mutation_field_name.as_str() => {
                        if let Some(data_value) =
                            context.args.get(&entity_update_mutation_data_field_name)
                        {
                            guard_data_object_accessor_with_optional_subscriber_id(
                                data_value,
                                &column_name,
                                subscriber_id,
                            )
                            .map_err(|inner_error| {
                                AuthError::from_graphql_dynamic_subscribe_id_guard(
                                    inner_error,
                                    context,
                                    &entity_update_mutation_data_field_name,
                                    &column_name,
                                )
                            })
                        } else {
                            Ok(())
                        }
                    }
                    _ => Ok(()),
                };
                match validation_result {
                    Ok(_) => GuardAction::Allow,
                    Err(err) => GuardAction::Block(Some(err.to_string())),
                }
            }
            Err(err) => GuardAction::Block(Some(err.message)),
        }
    })
}

pub fn generate_subscriber_id_filter_condition<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterCondition
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(
        move |context: &ResolverContext,
              mut condition: Condition,
              filter: Option<&ObjectAccessor<'_>>|
              -> SeaResult<Condition> {
            match context.ctx.data::<AuthUserInfo>() {
                Ok(user_info) => {
                    let subscriber_id = user_info.subscriber_auth.subscriber_id;

                    if let Some(filter) = filter {
                        for operation in &SUBSCRIBER_ID_FILTER_INFO.supported_operations {
                            match operation {
                                SeaographqlFilterOperation::Equals => {
                                    if let Some(value) = filter.get("eq") {
                                        let value: i32 = value.i64()?.try_into()?;
                                        if value != subscriber_id {
                                            return Err(SeaographyError::AsyncGraphQLError(
                                                async_graphql::Error::new(
                                                    "subscriber_id and auth_info does not match",
                                                ),
                                            ));
                                        }
                                    }
                                }
                                _ => unreachable!("unreachable filter operation for subscriber_id"),
                            }
                        }
                    } else {
                        condition = condition.add(column.eq(subscriber_id));
                    }

                    Ok(condition)
                }
                Err(err) => unreachable!("auth user info must be guarded: {:?}", err),
            }
        },
    )
}

pub fn generate_default_subscriber_id_input_conversion<T>(
    context: &BuilderContext,
    _column: &T::Column,
) -> FnInputTypeNoneConversion
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    let entity_name = context.entity_query_field.type_name.as_ref()(&entity_key);
    let entity_create_one_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name, context.entity_create_one_mutation.mutation_suffix
    ));
    let entity_create_batch_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name,
        context.entity_create_batch_mutation.mutation_suffix.clone()
    ));
    Box::new(
        move |context: &ResolverContext| -> SeaResult<Option<SeaValue>> {
            let field_name = context.field().name();
            tracing::warn!("field_name: {:?}", field_name);
            if field_name == entity_create_one_mutation_field_name.as_str()
                || field_name == entity_create_batch_mutation_field_name.as_str()
            {
                match context.ctx.data::<AuthUserInfo>() {
                    Ok(user_info) => {
                        let subscriber_id = user_info.subscriber_auth.subscriber_id;
                        Ok(Some(SeaValue::Int(Some(subscriber_id))))
                    }
                    Err(err) => unreachable!("auth user info must be guarded: {:?}", err),
                }
            } else {
                Ok(None)
            }
        },
    )
}

pub fn restrict_subscriber_for_entity<T>(context: &mut BuilderContext, column: &T::Column)
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    let entity_column_key = get_entity_column_key::<T>(context, column);

    context.guards.entity_guards.insert(
        entity_key.clone(),
        guard_entity_with_subscriber_id::<T>(context, column),
    );
    context.guards.field_guards.insert(
        entity_column_key.clone(),
        guard_field_with_subscriber_id::<T>(context, column),
    );
    context.filter_types.overwrites.insert(
        entity_column_key.clone(),
        Some(FilterType::Custom(
            SUBSCRIBER_ID_FILTER_INFO.type_name.clone(),
        )),
    );
    context.filter_types.condition_functions.insert(
        entity_column_key.clone(),
        generate_subscriber_id_filter_condition::<T>(context, column),
    );
    context.types.input_none_conversions.insert(
        entity_column_key.clone(),
        generate_default_subscriber_id_input_conversion::<T>(context, column),
    );

    context.entity_input.update_skips.push(entity_column_key);
}

pub fn register_subscribers_to_schema_context(context: &mut BuilderContext) {
    for column in subscribers::Column::iter() {
        if !matches!(column, subscribers::Column::Id) {
            let key = get_entity_column_key::<subscribers::Entity>(context, &column);
            context.filter_types.overwrites.insert(key, None);
        }
    }
}

pub fn register_subscribers_to_schema_builder(mut builder: SeaographyBuilder) -> SeaographyBuilder {
    {
        let filter_types_map_helper = FilterTypesMapHelper {
            context: builder.context,
        };

        builder.schema = builder
            .schema
            .register(filter_types_map_helper.generate_filter_input(&SUBSCRIBER_ID_FILTER_INFO));
    }

    {
        builder.register_entity::<subscribers::Entity>(
            <subscribers::RelatedEntity as sea_orm::Iterable>::iter()
                .map(|rel| seaography::RelationBuilder::get_relation(&rel, builder.context))
                .collect(),
        );
        builder = builder.register_entity_dataloader_one_to_one(subscribers::Entity, tokio::spawn);
        builder = builder.register_entity_dataloader_one_to_many(subscribers::Entity, tokio::spawn);
    }

    builder
}
