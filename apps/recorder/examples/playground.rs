#![allow(unused_imports)]
use color_eyre::eyre::Context;
use itertools::Itertools;
use loco_rs::{
    app::Hooks,
    boot::{BootResult, StartMode},
    environment::Environment,
    prelude::*,
};
use recorder::{
    app::App,
    extract::mikan::parse_mikan_rss_items_from_rss_link,
    migrations::Migrator,
    models::{
        subscribers::SEED_SUBSCRIBER,
        subscriptions::{self, SubscriptionCreateFromRssDto},
    },
};
use sea_orm_migration::MigratorTrait;

async fn pull_mikan_bangumi_rss(ctx: &AppContext) -> color_eyre::eyre::Result<()> {
    let rss_link = "https://mikanani.me/RSS/Bangumi?bangumiId=3416&subgroupid=370";

    // let rss_link =
    //     "https://mikanani.me/RSS/MyBangumi?token=FE9tccsML2nBPUUqpCuJW2uJZydAXCntHJ7RpD9LDP8%3d";
    let subscription = if let Some(subscription) = subscriptions::Entity::find()
        .filter(subscriptions::Column::SourceUrl.eq(String::from(rss_link)))
        .one(&ctx.db)
        .await?
    {
        subscription
    } else {
        subscriptions::Model::add_subscription(
            ctx,
            subscriptions::SubscriptionCreateDto::Mikan(SubscriptionCreateFromRssDto {
                rss_link: rss_link.to_string(),
                display_name: String::from("Mikan Project - 我的番组"),
                enabled: Some(true),
            }),
            1,
        )
        .await?
    };

    subscription.pull_subscription(ctx).await?;

    Ok(())
}

async fn init() -> color_eyre::eyre::Result<AppContext> {
    color_eyre::install()?;
    let ctx = loco_rs::cli::playground::<App>().await?;
    let BootResult {
        app_context: ctx, ..
    } = loco_rs::boot::run_app::<App>(&StartMode::ServerOnly, ctx).await?;
    Migrator::up(&ctx.db, None).await?;
    Ok(ctx)
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    let ctx = init().await?;
    pull_mikan_bangumi_rss(&ctx).await?;

    Ok(())
}
