//! Recorder configuration policy; shared parsing and interpolation live in
//! configuration.
use std::sync::OnceLock;

use konobangu_configuration::{ConfigSchema, Values};
pub use konobangu_configuration::{Snapshot, interpolate, tags};

use crate::errors::{RecorderError, RecorderResult};

pub static ENVIRONMENT: OnceLock<Snapshot> = OnceLock::new();
pub const SECRETS: &[&str] = &[
  "AUTH__PROVIDER__CREDENTIALS__PASSWORD",
  "AUTH__PROVIDER__SIGNING__PRIVATE_KEY_PEM",
  "AUTH__PROVIDER__DATABASE__URL",
  "AUTH__RESOURCE_GRANTS__SIGNING_KEY",
  "AUTH__IDENTITY__DATABASE_URL",
  "DATABASE__URL",
  "DATABASE__MIGRATION__URL",
  "SCHEDULER__DATABASE__URL",
];
const NAMESPACES: &[&str] = &[
  "SERVER",
  "CACHE",
  "AUTH",
  "STORAGE",
  "MIKAN",
  "CRYPTO",
  "GRAPHQL",
  "MEDIA",
  "LOGGER",
  "DATABASE",
  "SCHEDULER",
  "MESSAGE",
];

pub fn invalid(path: &str, category: &str) -> RecorderError {
  konobangu_configuration::invalid(path, category).into()
}

pub fn canonical(snapshot: &Snapshot) -> RecorderResult<Values> {
  konobangu_configuration::canonical(
    snapshot,
    &ConfigSchema {
      namespaces: NAMESPACES,
      secrets: SECRETS,
      removed_prefixes: &["TASK__"],
      removed_names: &["HOST", "DATABASE_URL"],
    },
  )
  .map_err(Into::into)
}
