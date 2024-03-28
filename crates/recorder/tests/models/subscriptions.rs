use insta::assert_debug_snapshot;
use loco_rs::testing;
use recorder::{
    app::App,
    models::{subscribers::ROOT_SUBSCRIBER_ID, subscriptions},
};
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
    testing::seed::<App>(&boot.app_context.db).await.unwrap();

    let create_rss = serde_json::from_str(
        r#"{
        "rss_link": "https://mikanani.me/RSS/Bangumi?bangumiId=3141&subgroupid=370",
        "display_name": "Mikan Project - 葬送的芙莉莲",
        "aggregate": false,
        "enabled": true,
        "category": "mikan"
    }"#,
    )
    .expect("should parse create rss dto from json");

    let subscription = subscriptions::ActiveModel::from_create_dto(create_rss, ROOT_SUBSCRIBER_ID)
        .await
        .expect("should create subscription");

    let subscription = subscriptions::ActiveModel::assert_debug_snapshot!(existing_subscriber);
}
