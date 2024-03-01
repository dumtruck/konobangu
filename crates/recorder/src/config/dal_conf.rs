use serde::{Deserialize, Serialize};

pub fn default_app_dal_fs_root() -> String {
    String::from("data")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDalConf {
    pub fs_root: String,
}
