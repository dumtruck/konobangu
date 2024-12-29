use serde::de::DeserializeOwned;

use crate::{
    dal::{config::AppDalConfig, DAL_CONF_KEY},
    extract::mikan::{AppMikanConfig, MIKAN_CONF_KEY},
};

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

    fn get_dal_conf(&self) -> loco_rs::Result<Option<AppDalConfig>> {
        deserialize_key_path_from_app_config(self.get_root_conf(), &[DAL_CONF_KEY])
    }

    fn get_mikan_conf(&self) -> loco_rs::Result<Option<AppMikanConfig>> {
        deserialize_key_path_from_app_config(self.get_root_conf(), &[MIKAN_CONF_KEY])
    }
}

impl AppConfigExt for loco_rs::config::Config {
    fn get_root_conf(&self) -> &loco_rs::config::Config {
        self
    }
}
