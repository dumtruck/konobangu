use figment::{
    providers::{Format, Json, Yaml},
    Figment,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{auth::AppAuthConfig, dal::config::AppDalConfig, extract::mikan::AppMikanConfig};

const DEFAULT_APP_SETTINGS_MIXIN: &str = include_str!("./settings_mixin.yaml");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub auth: AppAuthConfig,
    pub dal: AppDalConfig,
    pub mikan: AppMikanConfig,
}

pub fn deserialize_key_path_from_json_value<T: DeserializeOwned>(
    value: &serde_json::Value,
    key_path: &[&str],
) -> Result<Option<T>, loco_rs::Error> {
    let mut stack = vec![("", value)];
    for key in key_path {
        let current = stack.last().unwrap().1;
        if let Some(v) = current.get(key) {
            stack.push((key, v));
        } else {
            return Ok(None);
        }
    }
    let result: T = serde_json::from_value(stack.pop().unwrap().1.clone())?;
    Ok(Some(result))
}

pub fn deserialize_key_path_from_app_config<T: DeserializeOwned>(
    app_config: &loco_rs::config::Config,
    key_path: &[&str],
) -> Result<Option<T>, loco_rs::Error> {
    let settings = app_config.settings.as_ref();
    if let Some(settings) = settings {
        deserialize_key_path_from_json_value(settings, key_path)
    } else {
        Ok(None)
    }
}

pub trait AppConfigExt {
    fn get_root_conf(&self) -> &loco_rs::config::Config;

    fn get_app_conf(&self) -> loco_rs::Result<AppConfig> {
        let settings_str = self
            .get_root_conf()
            .settings
            .as_ref()
            .map(serde_json::to_string)
            .unwrap_or_else(|| Ok(String::new()))?;

        let app_config = Figment::from(Json::string(&settings_str))
            .merge(Yaml::string(DEFAULT_APP_SETTINGS_MIXIN))
            .extract()
            .map_err(loco_rs::Error::wrap)?;

        Ok(app_config)
    }
}

impl AppConfigExt for loco_rs::config::Config {
    fn get_root_conf(&self) -> &loco_rs::config::Config {
        self
    }
}
