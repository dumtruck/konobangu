use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    #[serde(default = "default_subscriber_task_workers")]
    pub subscriber_task_concurrency: u32,
    #[serde(default = "default_system_task_workers")]
    pub system_task_concurrency: u32,
    #[serde(default = "default_subscriber_task_timeout")]
    pub subscriber_task_timeout: Duration,
    #[serde(default = "default_system_task_timeout")]
    pub system_task_timeout: Duration,
    #[serde(default = "default_cron_retry_duration")]
    pub cron_retry_duration: Duration,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            subscriber_task_concurrency: default_subscriber_task_workers(),
            system_task_concurrency: default_system_task_workers(),
            subscriber_task_timeout: default_subscriber_task_timeout(),
            system_task_timeout: default_system_task_timeout(),
            cron_retry_duration: default_cron_retry_duration(),
        }
    }
}

pub fn default_subscriber_task_workers() -> u32 {
    if cfg!(test) {
        1
    } else {
        ((num_cpus::get_physical() as f32 / 2.0).floor() as u32).max(1)
    }
}

pub fn default_system_task_workers() -> u32 {
    if cfg!(test) {
        1
    } else {
        ((num_cpus::get_physical() as f32 / 2.0).floor() as u32).max(1)
    }
}

pub fn default_subscriber_task_timeout() -> Duration {
    Duration::from_secs(3600)
}

pub fn default_system_task_timeout() -> Duration {
    Duration::from_secs(3600)
}

pub fn default_cron_retry_duration() -> Duration {
    Duration::from_secs(5)
}
