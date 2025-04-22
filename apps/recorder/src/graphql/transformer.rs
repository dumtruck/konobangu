use async_graphql::dynamic::ResolverContext;
use sea_orm::{ColumnTrait, Condition, EntityTrait};
use seaography::{BuilderContext, FnFilterConditionsTransformer};

use crate::auth::AuthUserInfo;

pub fn filter_condition_transformer<T>(
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
