mod config;
mod core;
mod registry;
mod service;

pub use core::{SUBSCRIBER_TASK_APALIS_NAME, SubscriberAsyncTaskTrait, SubscriberStreamTaskTrait};

pub use config::TaskConfig;
pub use registry::{
    SubscriberTask, SubscriberTaskType, SubscriberTaskTypeEnum, SubscriberTaskTypeVariant,
    SubscriberTaskTypeVariantIter, SyncOneSubscriptionFeedsFullTask,
    SyncOneSubscriptionFeedsIncrementalTask, SyncOneSubscriptionSourcesTask,
};
pub use service::TaskService;
