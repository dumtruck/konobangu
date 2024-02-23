use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Subscribers {
    Table,
    Id,
    Pid,
    DisplayName,
}

#[derive(Iden)]
pub enum Subscriptions {
    Table,
    Id,
    SubscriberId,
    DisplayName,
    Category,
    SourceUrl,
    Aggregate,
    Enabled,
}

#[derive(Iden)]
pub enum Bangumi {
    Table,
    Id,
    DisplayName,
    SubscriptionId,
}

#[derive(Iden)]
pub enum Episodes {
    Table,
    Id,
    DisplayName,
    BangumiId,
    DownloadUrl,
    DownloadProgress,
    OutputName,
}
