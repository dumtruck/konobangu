mod base;
mod media;
mod misc;

pub(crate) use base::register_system_task_type;
pub use media::OptimizeImageTask;
pub use misc::EchoTask;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter, FromJsonQueryResult};

macro_rules! register_system_task_types {
    (
        task_type_enum: {
            $(#[$type_enum_meta:meta])*
            $type_vis:vis enum $type_enum_name:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $variant:ident => $string_value:literal
                ),* $(,)?
            }
        },
        task_enum: {
            $(#[$task_enum_meta:meta])*
            $task_vis:vis enum $task_enum_name:ident {
                $(
                    $(#[$task_variant_meta:meta])*
                    $task_variant:ident($task_type:ty)
                ),* $(,)?
            }
        }
    ) => {
        $(#[$type_enum_meta])*
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
        #[sea_orm(rs_type = "String", db_type = "Text")]
        $type_vis enum $type_enum_name {
            $(
                $(#[$variant_meta])*
                #[serde(rename = $string_value)]
                #[sea_orm(string_value = $string_value)]
                $variant,
            )*
        }


        $(#[$task_enum_meta])*
        #[derive(ts_rs::TS, serde::Serialize, serde::Deserialize, PartialEq)]
        #[serde(tag = "task_type")]
        #[ts(export, rename = "SystemTaskType", rename_all = "camelCase", tag = "taskType")]
        $task_vis enum $task_enum_name {
            $(
                $(#[$task_variant_meta])*
                #[serde(rename = $string_value)]
                $task_variant($task_type),
            )*
        }

        impl $task_enum_name {
            pub fn task_type(&self) -> $type_enum_name {
                match self {
                    $(Self::$task_variant(_) => $type_enum_name::$variant,)*
                }
            }
        }

        paste::paste! {
            $(#[$task_enum_meta])*
            #[derive(ts_rs::TS, serde::Serialize, serde::Deserialize, PartialEq)]
            #[serde(tag = "taskType", rename_all = "camelCase")]
            #[ts(export, rename_all = "camelCase", tag = "taskType")]
            $task_vis enum [<$task_enum_name Input>] {
                $(
                    $(#[$task_variant_meta])*
                    #[serde(rename = $string_value)]
                    $task_variant(<$task_type as $crate::task::SystemTaskTrait>::InputType),
                )*
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

        impl $crate::task::SystemTaskTrait for $task_enum_name {
            paste::paste! {
                type InputType = [<$task_enum_name Input>];
            }

            fn get_subscriber_id(&self) -> Option<i32> {
                match self {
                    $(Self::$task_variant(t) => t.get_subscriber_id(),)*
                }
            }

            fn get_cron_id(&self) -> Option<i32> {
                match self {
                    $(Self::$task_variant(t) => t.get_cron_id(),)*
                }
            }

            fn set_subscriber_id(&mut self, subscriber_id: Option<i32>) {
                match self {
                    $(Self::$task_variant(t) => t.set_subscriber_id(subscriber_id),)*
                }
            }

            fn set_cron_id(&mut self, cron_id: Option<i32>) {
                match self {
                    $(Self::$task_variant(t) => t.set_cron_id(cron_id),)*
                }
            }

            fn from_input(input: Self::InputType, subscriber_id: Option<i32>) -> Self {
                match input {
                    $(Self::InputType::$task_variant(t) =>
                        Self::$task_variant(<$task_type as $crate::task::SystemTaskTrait>::from_input(t, subscriber_id)),)*
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

register_system_task_types! {
    task_type_enum: {
        #[derive(
            Clone,
            Debug,
            Copy,
            DeriveActiveEnum,
            DeriveDisplay,
            EnumIter
        )]
        pub enum SystemTaskType {
            OptimizeImage => "optimize_image",
            Test => "test",
        }
    },
    task_enum: {
        #[derive(Clone, Debug, FromJsonQueryResult)]
        pub enum SystemTask {
            OptimizeImage(OptimizeImageTask),
            Echo(EchoTask),
        }
    }
}
