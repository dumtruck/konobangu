use std::{fs, path::Path, str};

use figment::{
    Figment, Provider,
    providers::{Format, Json, Toml, Yaml},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::env::Enviornment;
use crate::{
    auth::AuthConfig, errors::RResult, extract::mikan::AppMikanConfig,
    graphql::config::GraphQLConfig, storage::StorageConfig,
};

const DEFAULT_CONFIG_MIXIN: &str = include_str!("./default_mixin.toml");
const CONFIG_ALLOWED_EXTENSIONS: &[&str] = &[".toml", ".json", ".yaml", ".yml"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub auth: AuthConfig,
    pub dal: StorageConfig,
    pub mikan: AppMikanConfig,
    pub graphql: GraphQLConfig,
}

impl AppConfig {
    pub fn config_prefix() -> String {
        format!("{}.config", env!("CARGO_PKG_NAME"))
    }

    pub fn dotenv_prefix() -> String {
        String::from(".env")
    }

    pub fn allowed_extension() -> Vec<String> {
        CONFIG_ALLOWED_EXTENSIONS
            .iter()
            .map(|s| s.to_string())
            .collect_vec()
    }

    pub fn priority_suffix(enviornment: &Enviornment) -> Vec<String> {
        vec![
            format!(".{}.local", enviornment.full_name()),
            format!(".{}.local", enviornment.short_name()),
            String::from(".local"),
            enviornment.full_name().to_string(),
            enviornment.short_name().to_string(),
            String::from(""),
        ]
    }

    pub fn default_provider() -> impl Provider {
        Toml::string(DEFAULT_CONFIG_MIXIN)
    }

    pub fn merge_provider_from_file(
        fig: Figment,
        filepath: impl AsRef<Path>,
        ext: &str,
    ) -> RResult<Figment> {
        let content = fs::read_to_string(filepath)?;

        let rendered = tera::Tera::one_off(
            &content,
            &tera::Context::from_value(serde_json::json!({}))?,
            false,
        )?;

        Ok(match ext {
            ".toml" => fig.merge(Toml::string(&rendered)),
            ".json" => fig.merge(Json::string(&rendered)),
            ".yaml" | ".yml" => fig.merge(Yaml::string(&rendered)),
            _ => unreachable!("unsupported config extension"),
        })
    }
}
