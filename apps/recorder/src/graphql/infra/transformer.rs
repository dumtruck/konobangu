use std::{collections::BTreeMap, sync::Arc};

use async_graphql::dynamic::{ResolverContext, ValueAccessor};
use sea_orm::{ColumnTrait, Condition, EntityTrait, Value as SeaValue};
use seaography::{
    BuilderContext, FnFilterConditionsTransformer, FnMutationInputObjectTransformer, SeaResult,
};

use super::util::{get_column_key, get_entity_key};
use crate::{app::AppContextTrait, auth::AuthUserInfo, models::credential_3rd};

pub fn build_filter_condition_transformer<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterConditionsTransformer
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(
        move |context: &ResolverContext, condition: Condition| -> Condition {
            match context.ctx.data::<AuthUserInfo>() {
                Ok(user_info) => {
                    let subscriber_id = user_info.subscriber_auth.subscriber_id;
                    condition.add(column.eq(subscriber_id))
                }
                Err(err) => unreachable!("auth user info must be guarded: {:?}", err),
            }
        },
    )
}

pub fn build_mutation_input_object_transformer<T>(
    context: &BuilderContext,
    column: &T::Column,
) -> FnMutationInputObjectTransformer
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
    let entity_create_batch_mutation_field_name = Arc::new(format!(
        "{}{}",
        entity_name,
        context.entity_create_batch_mutation.mutation_suffix.clone()
    ));
    Box::new(
        move |context: &ResolverContext,
              mut input: BTreeMap<String, SeaValue>|
              -> BTreeMap<String, SeaValue> {
            let field_name = context.field().name();
            if field_name == entity_create_one_mutation_field_name.as_str()
                || field_name == entity_create_batch_mutation_field_name.as_str()
            {
                match context.ctx.data::<AuthUserInfo>() {
                    Ok(user_info) => {
                        let subscriber_id = user_info.subscriber_auth.subscriber_id;
                        let value = input.get_mut(column_name.as_str());
                        if value.is_none() {
                            input.insert(
                                column_name.as_str().to_string(),
                                SeaValue::Int(Some(subscriber_id)),
                            );
                        }
                        input
                    }
                    Err(err) => unreachable!("auth user info must be guarded: {:?}", err),
                }
            } else {
                input
            }
        },
    )
}

fn add_crypto_column_input_conversion<T>(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
    column: &T::Column,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    let column_name = get_column_key::<T>(context, column);
    let entity_name = context.entity_object.type_name.as_ref()(&entity_key);
    let column_name = context.entity_object.column_name.as_ref()(&entity_key, &column_name);

    context.types.input_conversions.insert(
        format!("{entity_name}.{column_name}"),
        Box::new(move |value: &ValueAccessor| -> SeaResult<sea_orm::Value> {
            let source = value.string()?;
            let encrypted = ctx.crypto().encrypt_string(source.into())?;
            Ok(encrypted.into())
        }),
    );
}

fn add_crypto_column_output_conversion<T>(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
    column: &T::Column,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_key = get_entity_key::<T>(context);
    let column_name = get_column_key::<T>(context, column);
    let entity_name = context.entity_object.type_name.as_ref()(&entity_key);
    let column_name = context.entity_object.column_name.as_ref()(&entity_key, &column_name);

    context.types.output_conversions.insert(
        format!("{entity_name}.{column_name}"),
        Box::new(
            move |value: &sea_orm::Value| -> SeaResult<async_graphql::Value> {
                if let SeaValue::String(s) = value {
                    if let Some(s) = s {
                        let decrypted = ctx.crypto().decrypt_string(s)?;
                        Ok(async_graphql::Value::String(decrypted))
                    } else {
                        Ok(async_graphql::Value::Null)
                    }
                } else {
                    Err(async_graphql::Error::new("crypto column must be string column").into())
                }
            },
        ),
    );
}

pub fn add_crypto_transformers(context: &mut BuilderContext, ctx: Arc<dyn AppContextTrait>) {
    add_crypto_column_input_conversion::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    add_crypto_column_input_conversion::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    add_crypto_column_input_conversion::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Password,
    );
    add_crypto_column_output_conversion::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    add_crypto_column_output_conversion::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    add_crypto_column_output_conversion::<credential_3rd::Entity>(
        context,
        ctx,
        &credential_3rd::Column::Password,
    );
}
