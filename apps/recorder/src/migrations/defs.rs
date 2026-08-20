pub use konobangu_database::migration::{CustomSchemaManagerExt, GeneralIds, table_auto_z, timestamps_z};
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum ApplicationSchema {
  Public,
  AuthIdentity,
}

#[derive(DeriveIden)]
pub enum Subscribers {
  Table,
  Id,
  DisplayName,
  DownloaderId,
  BangumiConf,
}

#[derive(DeriveIden)]
pub enum Subscriptions {
  Table,
  Id,
  DisplayName,
  SubscriberId,
  Category,
  SourceUrl,
  Enabled,
  CredentialId,
}

#[derive(DeriveIden)]
pub enum Bangumi {
  Table,
  Id,
  MikanBangumiId,
  DisplayName,
  SubscriberId,
  OriginName,
  Season,
  SeasonRaw,
  Fansub,
  MikanFansubId,
  Filter,
  RssLink,
  PosterLink,
  OriginPosterLink,
  /**
   * @deprecated
   */
  SavePath,
  Homepage,
  BangumiType,
}

#[derive(DeriveIden)]
pub enum SubscriptionBangumi {
  Table,
  Id,
  SubscriberId,
  SubscriptionId,
  BangumiId,
}

#[derive(DeriveIden)]
pub enum Episodes {
  Table,
  Id,
  MikanEpisodeId,
  OriginName,
  DisplayName,
  BangumiId,
  SubscriberId,
  DownloadId,
  /**
   * @deprecated
   */
  SavePath,
  Resolution,
  Season,
  SeasonRaw,
  Fansub,
  PosterLink,
  OriginPosterLink,
  EpisodeIndex,
  Homepage,
  Subtitle,
  Source,
  EpisodeType,
  EnclosureTorrentLink,
  EnclosureMagnetLink,
  EnclosurePubDate,
  EnclosureContentLength,
}

#[derive(DeriveIden)]
pub enum SubscriptionEpisode {
  Table,
  Id,
  SubscriberId,
  SubscriptionId,
  EpisodeId,
}

#[derive(DeriveIden)]
pub enum Downloads {
  Table,
  Id,
  OriginName,
  DisplayName,
  SubscriberId,
  DownloaderId,
  EpisodeId,
  Status,
  CurrSize,
  AllSize,
  Mime,
  Url,
  Homepage,
  SavePath,
}

#[derive(DeriveIden)]
pub enum Downloaders {
  Table,
  Id,
  Category,
  Endpoint,
  Password,
  Username,
  SubscriberId,
  SavePath,
}

#[derive(DeriveIden)]
pub enum Auth {
  Table,
  Id,
  Pid,
  SubscriberId,
  AuthType,
}

#[derive(DeriveIden)]
pub enum Credential3rd {
  Table,
  Id,
  SubscriberId,
  CredentialType,
  Cookies,
  Username,
  Password,
  UserAgent,
}

#[derive(DeriveIden)]
pub enum Feeds {
  Table,
  Id,
  Token,
  FeedType,
  FeedSource,
  SubscriberId,
  SubscriptionId,
}

#[derive(DeriveIden)]
pub enum Cron {
  Table,
  Id,
  SubscriberId,
  SubscriptionId,
  CronExpr,
  CronTimezone,
  NextRun,
  LastRun,
  LastError,
  Enabled,
  SubscriberTaskCron,
  SystemTaskCron,
}

#[derive(sea_query::Iden)]

pub enum ApalisSchema {
  #[iden = "apalis"]
  Schema,
}

// Application-owned delivery identifiers are shared by DDL and migration
// queries.
#[derive(DeriveIden)]
pub enum TaskRuns {
  Table,
  Id,
  Kind,
  SubscriberId,
  SubscriptionId,
  CronId,
  ScheduledAtUtc,
  PayloadVersion,
  Payload,
  TaskType,
  Status,
  Generation,
  Attempts,
  MaxAttempts,
  RunAt,
  DoneAt,
  LastError,
  CancelRequestedAt,
  ArchivedAt,
  ExecutionToken,
  ExecutionLeaseUntil,
  CreatedAt,
}

#[derive(DeriveIden)]
pub enum TaskOutbox {
  Table,
  TaskId,
  Generation,
  AvailableAt,
  LeaseToken,
  LeaseUntil,
  Attempts,
  LastError,
  SentAt,
}

#[derive(DeriveIden)]
pub enum TaskViews {
  SubscriberTasks,
  SystemTasks,
  Job,
}

macro_rules! create_postgres_enum_for_active_enum {
    ($manager: expr, $active_enum: expr, $($enum_value:expr),+) => {
        {
            use sea_orm::ActiveEnum;
            let values = [$($enum_value,)+].map(|v| ActiveEnum::to_value(&v));
            ($manager).create_postgres_enum_for_active_enum($active_enum, values)
        }
    };
}
