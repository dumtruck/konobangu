use std::{fs, path::Path, str};

use figment::{
    Figment, Provider,
    providers::{Format, Json, Toml, Yaml},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::env::Environment;
use crate::{
    auth::AuthConfig, cache::CacheConfig, crypto::CryptoConfig, database::DatabaseConfig,
    errors::RecorderResult, extract::mikan::MikanConfig, graphql::GraphQLConfig,
    logger::LoggerConfig, media::MediaConfig, message::MessageConfig, storage::StorageConfig,
    task::TaskConfig, web::WebServerConfig,
};

const DEFAULT_CONFIG_MIXIN: &str = include_str!("./default_mixin.toml");
const CONFIG_ALLOWED_EXTENSIONS: &[&str] = &[".toml", ".json", ".yaml", ".yml"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: WebServerConfig,
    pub cache: CacheConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub mikan: MikanConfig,
    pub crypto: CryptoConfig,
    pub graphql: GraphQLConfig,
    pub media: MediaConfig,
    pub logger: LoggerConfig,
    pub database: DatabaseConfig,
    pub task: TaskConfig,
    pub message: MessageConfig,
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

    pub fn priority_suffix(environment: &Environment) -> Vec<String> {
        vec![
            format!(".{}.local", environment.full_name()),
            format!(".{}.local", environment.short_name()),
            String::from(".local"),
            format!(".{}", environment.full_name()),
            format!(".{}", environment.short_name()),
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
    ) -> RecorderResult<Figment> {
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

    pub async fn load_dotenv(
        environment: &Environment,
        dotenv_file: Option<&str>,
    ) -> RecorderResult<()> {
        let try_dotenv_file_or_dirs = if dotenv_file.is_some() {
            vec![dotenv_file]
        } else {
            vec![Some(".")]
        };

        let priority_suffix = &AppConfig::priority_suffix(environment);
        let dotenv_prefix = AppConfig::dotenv_prefix();
        let try_filenames = priority_suffix
            .iter()
            .map(|ps| format!("{}{}", &dotenv_prefix, ps))
            .collect_vec();

        for try_dotenv_file_or_dir in try_dotenv_file_or_dirs.into_iter().flatten() {
            let try_dotenv_file_or_dir_path = Path::new(try_dotenv_file_or_dir);
            if try_dotenv_file_or_dir_path.exists() {
                if try_dotenv_file_or_dir_path.is_dir() {
                    for f in try_filenames.iter() {
                        let p = try_dotenv_file_or_dir_path.join(f);
                        if p.exists() && p.is_file() {
                            println!("Loading dotenv file: {}", p.display());
                            dotenvy::from_path(p)?;
                            break;
                        }
                    }
                } else if try_dotenv_file_or_dir_path.is_file() {
                    println!(
                        "Loading dotenv file: {}",
                        try_dotenv_file_or_dir_path.display()
                    );
                    dotenvy::from_path(try_dotenv_file_or_dir_path)?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn load_config(
        environment: &Environment,
        config_file: Option<&str>,
    ) -> RecorderResult<AppConfig> {
        let try_config_file_or_dirs = if config_file.is_some() {
            vec![config_file]
        } else {
            vec![Some(".")]
        };

        let allowed_extensions = &AppConfig::allowed_extension();
        let priority_suffix = &AppConfig::priority_suffix(environment);
        let convention_prefix = &AppConfig::config_prefix();

        let try_filenames = priority_suffix
            .iter()
            .flat_map(|ps| {
                allowed_extensions
                    .iter()
                    .map(move |ext| (format!("{convention_prefix}{ps}{ext}"), ext))
            })
            .collect_vec();

        let mut fig = Figment::from(AppConfig::default_provider());

        for try_config_file_or_dir in try_config_file_or_dirs.into_iter().flatten() {
            let try_config_file_or_dir_path = Path::new(try_config_file_or_dir);
            if try_config_file_or_dir_path.exists() {
                if try_config_file_or_dir_path.is_dir() {
                    for (f, ext) in try_filenames.iter() {
                        let p = try_config_file_or_dir_path.join(f);
                        if p.exists() && p.is_file() {
                            fig = AppConfig::merge_provider_from_file(fig, &p, ext)?;
                            println!("Loaded config file: {}", p.display());
                            break;
                        }
                    }
                } else if let Some(ext) = try_config_file_or_dir_path
                    .extension()
                    .and_then(|s| s.to_str())
                    && try_config_file_or_dir_path.is_file()
                {
                    fig =
                        AppConfig::merge_provider_from_file(fig, try_config_file_or_dir_path, ext)?;
                    println!(
                        "Loaded config file: {}",
                        try_config_file_or_dir_path.display()
                    );
                    break;
                }
            }
        }

        let app_config: AppConfig = fig.extract()?;

        Ok(app_config)
    }
}
