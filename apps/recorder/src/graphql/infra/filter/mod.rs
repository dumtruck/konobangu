mod json;
mod subscriber;

use async_graphql::dynamic::TypeRef;
pub use json::{
    JSONB_FILTER_NAME, jsonb_filter_condition_function,
    register_jsonb_input_filter_to_dynamic_schema,
};
use maplit::btreeset;
use seaography::{FilterInfo, FilterOperation as SeaographqlFilterOperation};
pub use subscriber::{SUBSCRIBER_ID_FILTER_INFO, subscriber_id_condition_function};

pub fn init_custom_filter_info() {
    SUBSCRIBER_ID_FILTER_INFO.get_or_init(|| FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    });
}
