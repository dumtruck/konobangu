use serde::{Deserialize, Serialize};

pub const CRON_DUE_EVENT: &str = "cron_due";

pub const CHECK_AND_CLEANUP_EXPIRED_CRON_LOCKS_FUNCTION_NAME: &str =
    "check_and_cleanup_expired_cron_locks";
pub const CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME: &str = "check_and_trigger_due_crons";

pub const NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME: &str = "notify_due_cron_when_mutating";
pub const NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME: &str =
    "notify_due_cron_when_mutating_trigger";
pub const SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME: &str = "setup_cron_extra_foreign_keys";
pub const SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME: &str =
    "setup_cron_extra_foreign_keys_trigger";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CronCreateOptions {
    pub cron_expr: String,
    pub priority: Option<i32>,
    pub timeout_ms: Option<i32>,
    pub max_attempts: Option<i32>,
    pub enabled: Option<bool>,
}
