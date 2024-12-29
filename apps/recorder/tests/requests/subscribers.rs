#![allow(unused_imports)]
use insta::{assert_debug_snapshot, with_settings};
use loco_rs::testing;
use recorder::app::App;
use serial_test::serial;

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
// macro_rules! configure_insta {
//     ($($expr:expr),*) => {
//         let mut settings = insta::Settings::clone_current();
//         settings.set_prepend_module_to_snapshot(false);
//         settings.set_snapshot_suffix("user_request");
//         let _guard = settings.bind_to_scope();
//     };
// }

#[tokio::test]
#[serial]
async fn can_get_current_user() {
    // configure_insta!();
    //
    // testing::request::<App, _, _>(|request, _ctx| async move {
    //     let response = request.get("/api/user/current").await;
    //
    //     with_settings!({
    //         filters => testing::cleanup_user_model()
    //     }, {
    //         assert_debug_snapshot!((response.status_code(),
    // response.text()));     });
    // })
    // .await;
}
