use std::sync::Arc;

use async_graphql::dynamic::{ResolverContext, ValueAccessor};
use sea_orm::{EntityTrait, Value as SeaValue};
use seaography::{BuilderContext, SeaResult};

use crate::{app::AppContextTrait, graphql::infra::name::get_entity_and_column_name};

pub fn register_crypto_column_input_conversion_to_schema_context<T>(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
    column: &T::Column,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    context.types.input_conversions.insert(
        get_entity_and_column_name::<T>(context, column),
        Arc::new(
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

pub fn register_crypto_column_output_conversion_to_schema_context<T>(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
    column: &T::Column,
) where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    context.types.output_conversions.insert(
        get_entity_and_column_name::<T>(context, column),
        Arc::new(
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
