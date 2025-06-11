use async_graphql::dynamic::ObjectAccessor;
use sea_orm::Condition;
use seaography::SeaResult;

pub type FnFilterCondition =
    Box<dyn Fn(Condition, &ObjectAccessor) -> SeaResult<Condition> + Send + Sync>;
