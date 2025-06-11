use async_graphql::dynamic::TypeRef;
use lazy_static::lazy_static;
use maplit::btreeset;
use sea_orm::{ColumnTrait, EntityTrait};
use seaography::{BuilderContext, FilterInfo, FilterOperation as SeaographqlFilterOperation};

use crate::graphql::infra::filter::FnFilterCondition;

lazy_static! {
    pub static ref SUBSCRIBER_ID_FILTER_INFO: FilterInfo = FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    };
}

pub fn generate_subscriber_id_condition_function<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterCondition
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(move |mut condition, filter| {
        for operation in &SUBSCRIBER_ID_FILTER_INFO.supported_operations {
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
