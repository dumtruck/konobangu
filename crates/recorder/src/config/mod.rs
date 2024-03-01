pub mod dal_conf;
pub use dal_conf::AppDalConf;
use eyre::OptionExt;
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const DAL_CONF_KEY: &str = "dal";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppCustomConf {
    pub dal: AppDalConf,
}

pub fn deserialize_key_path_from_json_value<T: DeserializeOwned>(
    key_path: &[&str],
    value: &serde_json::Value,
) -> eyre::Result<T> {
    let mut stack = vec![("", value)];
    for key in key_path {
        let current = stack.last().unwrap().1;
        if let Some(v) = current.get(key) {
            stack.push((key, v));
        } else {
            let failed_key_path = stack.iter().map(|s| s.0).collect_vec().join(".");
            return Err(eyre::eyre!(
                "can not config key {} of settings",
                failed_key_path
            ));
        }
    }
    let result: T = serde_json::from_value(stack.pop().unwrap().1.clone())?;
    Ok(result)
}

pub fn deserialize_key_path_from_loco_rs_config<T: DeserializeOwned>(
    key_path: &[&str],
    app_config: &loco_rs::config::Config,
) -> eyre::Result<T> {
    let settings = app_config
        .settings
        .as_ref()
        .ok_or_eyre("App config setting not set")?;
    deserialize_key_path_from_json_value(key_path, settings)
}
