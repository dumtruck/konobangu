use std::sync::Arc;

use async_graphql::dynamic::{ResolverContext, ValueAccessor};
use sea_orm::{EntityTrait, Value as SeaValue};
use seaography::{BuilderContext, SeaResult};

use crate::{
    app::AppContextTrait,
    graphql::infra::util::{get_column_key, get_entity_key},
    models::credential_3rd,
};

fn register_crypto_column_input_conversion_to_schema_context<T>(
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
        Box::new(
            move |_resolve_context: &ResolverContext<'_>,
                  value: &ValueAccessor|
                  -> SeaResult<sea_orm::Value> {
                let source = value.string()?;
                let encrypted = ctx.crypto().encrypt_string(source.into())?;
                Ok(encrypted.into())
            },
        ),
    );
}

fn register_crypto_column_output_conversion_to_schema_context<T>(
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

pub fn register_crypto_to_schema_context(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
) {
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Password,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx,
        &credential_3rd::Column::Password,
    );
}
