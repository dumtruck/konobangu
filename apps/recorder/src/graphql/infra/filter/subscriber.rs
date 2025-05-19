use async_graphql::dynamic::ObjectAccessor;
use once_cell::sync::OnceCell;
use sea_orm::{ColumnTrait, Condition, EntityTrait};
use seaography::{
    BuilderContext, FilterInfo, FilterOperation as SeaographqlFilterOperation, SeaResult,
};

pub static SUBSCRIBER_ID_FILTER_INFO: OnceCell<FilterInfo> = OnceCell::new();

pub type FnFilterCondition =
    Box<dyn Fn(Condition, &ObjectAccessor) -> SeaResult<Condition> + Send + Sync>;

pub fn subscriber_id_condition_function<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterCondition
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(move |mut condition, filter| {
        let subscriber_id_filter_info = SUBSCRIBER_ID_FILTER_INFO.get().unwrap();
        let operations = &subscriber_id_filter_info.supported_operations;
        for operation in operations {
            match operation {
                SeaographqlFilterOperation::Equals => {
                    if let Some(value) = filter.get("eq") {
                        let value: i32 = value.i64()?.try_into()?;
                        let value = sea_orm::Value::Int(Some(value));
                        condition = condition.add(column.eq(value));
                    }
                }
                _ => unreachable!("unreachable filter operation for subscriber_id"),
            }
        }
        Ok(condition)
    })
}
