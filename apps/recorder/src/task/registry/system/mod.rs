mod media;

pub use media::OptimizeImageTask;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

macro_rules! register_system_task_types {
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
    };
}

register_system_task_types! {
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
        pub enum SystemTaskType {
            OptimizeImage => "optimize_image"
        }
    },
    task_enum: {
        #[derive(Clone, Debug, Serialize, Deserialize,  FromJsonQueryResult)]
        pub enum SystemTask {
            OptimizeImage(OptimizeImageTask),
        }
    }
}
