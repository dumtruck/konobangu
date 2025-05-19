mod json;
mod subscriber;

use std::borrow::Cow;

use async_graphql::dynamic::TypeRef;
pub use json::{JSONB_FILTER_INFO, jsonb_filter_condition_function};
use maplit::btreeset;
use seaography::{FilterInfo, FilterOperation as SeaographqlFilterOperation};
pub use subscriber::{SUBSCRIBER_ID_FILTER_INFO, subscriber_id_condition_function};

pub fn init_custom_filter_info() {
    SUBSCRIBER_ID_FILTER_INFO.get_or_init(|| FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    });
    JSONB_FILTER_INFO.get_or_init(|| FilterInfo {
        type_name: String::from("JsonbFilterInput"),
        base_type: TypeRef::Named(Cow::Borrowed("serde_json::Value")).to_string(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    });
}
