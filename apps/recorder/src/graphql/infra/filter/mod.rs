mod json;

use async_graphql::{
    InputValueResult, Scalar, ScalarType,
    dynamic::{ObjectAccessor, TypeRef},
};
pub use json::recursive_prepare_json_node_condition;
use maplit::btreeset;
use once_cell::sync::OnceCell;
use sea_orm::{ColumnTrait, Condition, EntityTrait};
use seaography::{
    BuilderContext, FilterInfo, FilterOperation as SeaographqlFilterOperation, SeaResult,
};

pub static SUBSCRIBER_ID_FILTER_INFO: OnceCell<FilterInfo> = OnceCell::new();

pub fn init_custom_filter_info() {
    SUBSCRIBER_ID_FILTER_INFO.get_or_init(|| FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    });
}

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

#[derive(Clone, Debug)]
pub struct JsonFilterInput(pub serde_json::Value);

#[Scalar(name = "JsonFilterInput")]
impl ScalarType for JsonFilterInput {
    fn parse(value: async_graphql::Value) -> InputValueResult<Self> {
        Ok(JsonFilterInput(value.into_json()?))
    }

    fn to_value(&self) -> async_graphql::Value {
        async_graphql::Value::from_json(self.0.clone()).unwrap()
    }
}
