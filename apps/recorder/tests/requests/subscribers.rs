#![allow(unused_imports)]
use insta::{assert_debug_snapshot, with_settings};
use serial_test::serial;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("user_request");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_get_current_user() {
    configure_insta!();
}
