use recorder::errors::RResult;
// #![allow(unused_imports)]
// use recorder::{
//     app::{AppContext, AppContextTrait},
//     errors::RResult,
//     migrations::Migrator,
//     models::{
//         subscribers::SEED_SUBSCRIBER,
//         subscriptions::{self, SubscriptionCreateFromRssDto},
//     },
// };
// use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
// use sea_orm_migration::MigratorTrait;

// async fn pull_mikan_bangumi_rss(ctx: &dyn AppContextTrait) -> RResult<()> {
//     let rss_link = "https://mikanani.me/RSS/Bangumi?bangumiId=3416&subgroupid=370";

//     // let rss_link =
//     //     "https://mikanani.me/RSS/MyBangumi?token=FE9tccsML2nBPUUqpCuJW2uJZydAXCntHJ7RpD9LDP8%3d";
//     let subscription = if let Some(subscription) =
// subscriptions::Entity::find()
//         .filter(subscriptions::Column::SourceUrl.eq(String::from(rss_link)))
//         .one(ctx.db())
//         .await?
//     {
//         subscription
//     } else {
//         subscriptions::Model::add_subscription(
//             ctx,
//
// subscriptions::SubscriptionCreateDto::Mikan(SubscriptionCreateFromRssDto {
//                 rss_link: rss_link.to_string(),
//                 display_name: String::from("Mikan Project - 我的番组"),
//                 enabled: Some(true),
//             }),
//             1,
//         )
//         .await?
//     };

//     subscription.pull_subscription(ctx).await?;

//     Ok(())
// }

// #[tokio::main]
// async fn main() -> RResult<()> {
//     pull_mikan_bangumi_rss(&ctx).await?;

//     Ok(())
// }

#[tokio::main]
async fn main() -> RResult<()> {
    Ok(())
}
