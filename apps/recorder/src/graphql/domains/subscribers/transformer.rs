use std::{collections::BTreeMap, sync::Arc};

use async_graphql::dynamic::ResolverContext;
use sea_orm::{ColumnTrait, Condition, EntityTrait, Value as SeaValue};
use seaography::{BuilderContext, FnFilterConditionsTransformer, FnMutationInputObjectTransformer};

use crate::{
    auth::AuthUserInfo,
    graphql::infra::util::{get_column_key, get_entity_key},
};

pub fn generate_subscriber_id_filter_condition_transformer<T>(
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

pub fn generate_subscriber_id_mutation_input_object_transformer<T>(
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
