macro_rules! register_subscriber_task_type {
    (
        $(#[$type_meta:meta])*
        $task_vis:vis struct $task_name:ident {
            $($(#[$field_meta:meta])* pub $field_name:ident: $field_type:ty),* $(,)?
        }
    ) => {
        $(#[$type_meta])*
        #[derive(typed_builder::TypedBuilder, schemars::JsonSchema, ts_rs::TS)]
        #[ts(export, rename_all = "camelCase")]
        $task_vis struct $task_name {
            $($(#[$field_meta])* pub $field_name: $field_type,)*
            pub subscriber_id: i32,
            #[builder(default = None)]
            pub cron_id: Option<i32>,
        }

        impl $crate::task::SubscriberTaskTrait for $task_name {
            fn get_subscriber_id(&self) -> i32 {
                self.subscriber_id
            }

            fn get_cron_id(&self) -> Option<i32> {
                self.cron_id
            }
        }
    }
}

pub(crate) use register_subscriber_task_type;
