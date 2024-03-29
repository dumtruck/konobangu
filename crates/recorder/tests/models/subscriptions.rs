// use insta::assert_debug_snapshot;
use loco_rs::{app::Hooks, testing};
use recorder::{
    app::App,
    models::{
        subscribers::{self},
        subscriptions,
    },
};
use sea_orm::{ActiveModelTrait, TryIntoModel};
use serial_test::serial;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("subscriptions");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_pull_subscription() {
    configure_insta!();

    let boot = testing::boot_test::<App>().await.unwrap();
    App::init_logger(&boot.app_context.config, &boot.app_context.environment).unwrap();
    testing::seed::<App>(&boot.app_context.db).await.unwrap();
    let db = &boot.app_context.db;

    let create_rss = serde_json::from_str(
        r#"{
        "rss_link": "https://mikanani.me/RSS/Bangumi?bangumiId=3271&subgroupid=370",
        "display_name": "Mikan Project - 我心里危险的东西 第二季",
        "aggregate": false,
        "enabled": true,
        "category": "mikan"
      }"#,
    )
    .expect("should parse create rss dto from json");

    let subscriber = subscribers::Model::find_by_pid(db, subscribers::ROOT_SUBSCRIBER_NAME)
        .await
        .expect("should find subscriber");

    let subscription = subscriptions::ActiveModel::from_create_dto(create_rss, subscriber.id);

    let subscription = subscription
        .save(&boot.app_context.db)
        .await
        .expect("should save subscription")
        .try_into_model()
        .expect("should convert to model");

    subscription
        .pull_one(&boot.app_context, &subscriber)
        .await
        .expect("should pull subscription");

    // assert_debug_snapshot!(a);
}
