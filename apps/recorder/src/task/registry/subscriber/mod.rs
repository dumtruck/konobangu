mod base;
mod subscription;

use jsonschema::Validator;
use once_cell::sync::OnceCell;
use schemars::JsonSchema;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
pub use subscription::{
    SyncOneSubscriptionFeedsFullTask, SyncOneSubscriptionFeedsIncrementalTask,
    SyncOneSubscriptionSourcesTask,
};

macro_rules! register_subscriber_task_types {
    (
        task_type_enum: {
            $(#[$type_enum_meta:meta])*
            pub enum $type_enum_name:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $variant:ident => $string_value:literal
                ),* $(,)?
            }
        },
        task_enum: {
            $(#[$task_enum_meta:meta])*
            pub enum $task_enum_name:ident {
                $(
                    $(#[$task_variant_meta:meta])*
                    $task_variant:ident($task_type:ty)
                ),* $(,)?
            }
        }
    ) => {
        $(#[$type_enum_meta])*
        #[sea_orm(rs_type = "String", db_type = "Text")]
        pub enum $type_enum_name {
            $(
                $(#[$variant_meta])*
                #[serde(rename = $string_value)]
                #[sea_orm(string_value = $string_value)]
                $variant,
            )*
        }


        $(#[$task_enum_meta])*
        #[serde(tag = "task_type")]
        pub enum $task_enum_name {
            $(
                $(#[$task_variant_meta])*
                #[serde(rename = $string_value)]
                $task_variant($task_type),
            )*
        }

        impl TryFrom<$task_enum_name> for serde_json::Value {
            type Error = $crate::errors::RecorderError;

            fn try_from(value: $task_enum_name) -> Result<Self, Self::Error> {
                let json_value = serde_json::to_value(value)?;
                Ok(match json_value {
                    serde_json::Value::Object(mut map) => {
                        map.remove("task_type");
                        serde_json::Value::Object(map)
                    }
                    _ => {
                        unreachable!("subscriber task must be an json object");
                    }
                })
            }
        }

        impl $task_enum_name {
            pub fn task_type(&self) -> $type_enum_name {
                match self {
                    $(Self::$task_variant(_) => $type_enum_name::$variant,)*
                }
            }
        }

        #[async_trait::async_trait]
        impl $crate::task::AsyncTaskTrait for $task_enum_name {
            async fn run_async(self, ctx: std::sync::Arc<dyn $crate::app::AppContextTrait>) -> $crate::errors::RecorderResult<()> {
                match self {
                    $(Self::$task_variant(t) =>
                        <$task_type as $crate::task::AsyncTaskTrait>::run_async(t, ctx).await,)*
                }
            }
        }

        impl $crate::task::SubscriberTaskTrait for $task_enum_name {
            fn get_subscriber_id(&self) -> i32 {
                match self {
                    $(Self::$task_variant(t) =>
                        <$task_type as $crate::task::SubscriberTaskTrait>::get_subscriber_id(t),)*
                }
            }

            fn get_cron_id(&self) -> Option<i32> {
                match self {
                    $(Self::$task_variant(t) =>
                        <$task_type as $crate::task::SubscriberTaskTrait>::get_cron_id(t),)*
                }
            }
        }

        $(
            impl From<$task_type> for $task_enum_name {
                fn from(task: $task_type) -> Self {
                    Self::$task_variant(task)
                }
            }
        )*
    };
}

register_subscriber_task_types!(
    task_type_enum: {
        #[derive(
            Clone,
            Debug,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            Copy,
            DeriveActiveEnum,
            DeriveDisplay,
            EnumIter,
        )]
        pub enum SubscriberTaskType {
            SyncOneSubscriptionFeedsIncremental => "sync_one_subscription_feeds_incremental",
            SyncOneSubscriptionFeedsFull => "sync_one_subscription_feeds_full",
            SyncOneSubscriptionSources => "sync_one_subscription_sources"
        }
    },
    task_enum: {
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, FromJsonQueryResult, JsonSchema)]
        pub enum SubscriberTask {
            SyncOneSubscriptionFeedsIncremental(SyncOneSubscriptionFeedsIncrementalTask),
            SyncOneSubscriptionFeedsFull(SyncOneSubscriptionFeedsFullTask),
            SyncOneSubscriptionSources(SyncOneSubscriptionSourcesTask),
        }
    }
);

static SUBSCRIBER_TASK_SCHEMA: OnceCell<Validator> = OnceCell::new();

pub fn subscriber_task_schema() -> &'static Validator {
    SUBSCRIBER_TASK_SCHEMA.get_or_init(|| {
        let schema = schemars::schema_for!(SubscriberTask);
        jsonschema::options()
            .with_draft(jsonschema::Draft::Draft7)
            .build(&serde_json::to_value(&schema).unwrap())
            .unwrap()
    })
}
