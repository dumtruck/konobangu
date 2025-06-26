mod config;
mod core;
mod r#extern;
mod registry;
mod service;

pub use core::{
    AsyncTaskTrait, SUBSCRIBER_TASK_APALIS_NAME, SYSTEM_TASK_APALIS_NAME, StreamTaskTrait,
    SubscriberTaskBase, SubscriberTaskTrait, SystemTaskTrait,
};

pub use config::TaskConfig;
pub use r#extern::{ApalisJobs, ApalisSchema};
pub use registry::{
    OptimizeImageTask, SubscriberTask, SubscriberTaskType, SubscriberTaskTypeEnum,
    SubscriberTaskTypeVariant, SubscriberTaskTypeVariantIter, SyncOneSubscriptionFeedsFullTask,
    SyncOneSubscriptionFeedsIncrementalTask, SyncOneSubscriptionSourcesTask, SystemTask,
    SystemTaskType, SystemTaskTypeEnum, SystemTaskTypeVariant, SystemTaskTypeVariantIter,
};
pub use service::TaskService;
