mod resolve;

use std::{fs, path::Path};

use figment::{
  Figment, Provider,
  providers::{Format, Json, Toml, YamlExtended},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::env::Environment;
use crate::{
  auth::{AuthConfig, ProviderAuthConfig},
  cache::CacheConfig,
  crypto::CryptoConfig,
  database::DatabaseConfig,
  errors::RecorderResult,
  extract::mikan::MikanConfig,
  graphql::GraphQLConfig,
  logger::LoggerConfig,
  media::MediaConfig,
  message::MessageConfig,
  storage::StorageConfig,
  task::TaskConfig,
  web::WebServerConfig,
};

const DEFAULT_CONFIG_MIXIN: &str = include_str!("./default_mixin.toml");
const CONFIG_ALLOWED_EXTENSIONS: &[&str] = &[".toml", ".json", ".yaml", ".yml"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
  pub server: WebServerConfig,
  #[serde(skip)]
  pub cache: CacheConfig,
  pub auth: AuthConfig,
  pub storage: StorageConfig,
  pub mikan: MikanConfig,
  #[serde(skip)]
  pub crypto: CryptoConfig,
  pub graphql: GraphQLConfig,
  pub media: MediaConfig,
  pub logger: LoggerConfig,
  pub database: DatabaseConfig,
  #[serde(rename = "scheduler")]
  pub task: TaskConfig,
  #[serde(skip)]
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
    CONFIG_ALLOWED_EXTENSIONS.iter().map(|s| s.to_string()).collect_vec()
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

  pub fn merge_provider_from_file(fig: Figment, filepath: impl AsRef<Path>, ext: &str) -> RecorderResult<Figment> {
    let content = fs::read_to_string(filepath)?;
    Ok(match ext {
      ".toml" => fig.merge(Toml::string(&content)),
      ".json" => fig.merge(Json::string(&content)),
      ".yaml" | ".yml" => fig.merge(YamlExtended::string(&content)),
      _ => {
        return Err(crate::errors::RecorderError::InvalidConfiguration {
          message: "Unsupported configuration format; expected TOML, JSON or YAML".into(),
        });
      }
    })
  }

  fn merge_environment(fig: Figment, snapshot: &resolve::Snapshot) -> RecorderResult<Figment> {
    Ok(fig.merge(resolve::canonical(snapshot)?))
  }

  pub async fn load_dotenv(environment: &Environment, dotenv_file: Option<&str>) -> RecorderResult<()> {
    if resolve::ENVIRONMENT.get().is_some() {
      return Ok(());
    }
    let mut snapshot = resolve::Snapshot::new();
    let try_dotenv_file_or_dirs = if dotenv_file.is_some() { vec![dotenv_file] } else { vec![Some(".")] };

    let priority_suffix = &AppConfig::priority_suffix(environment);
    let dotenv_prefix = AppConfig::dotenv_prefix();
    let try_filenames = priority_suffix.iter().map(|ps| format!("{}{}", dotenv_prefix, ps)).collect_vec();

    for try_dotenv_file_or_dir in try_dotenv_file_or_dirs.into_iter().flatten() {
      let try_dotenv_file_or_dir_path = Path::new(try_dotenv_file_or_dir);
      if try_dotenv_file_or_dir_path.exists() {
        if try_dotenv_file_or_dir_path.is_dir() {
          for f in try_filenames.iter() {
            let p = try_dotenv_file_or_dir_path.join(f);
            if p.exists() && p.is_file() {
              for entry in dotenvy::from_path_iter(p).map_err(|_| resolve::invalid("dotenv", "invalid dotenv"))? {
                let (key, value) = entry.map_err(|_| resolve::invalid("dotenv", "invalid dotenv"))?;
                snapshot.insert(key, value);
              }
              break;
            }
          }
        } else if try_dotenv_file_or_dir_path.is_file() {
          for entry in dotenvy::from_path_iter(try_dotenv_file_or_dir_path).map_err(|_| resolve::invalid("dotenv", "invalid dotenv"))? {
            let (key, value) = entry.map_err(|_| resolve::invalid("dotenv", "invalid dotenv"))?;
            snapshot.insert(key, value);
          }
          break;
        }
      }
    }

    snapshot.extend(std::env::vars());
    let _ = resolve::ENVIRONMENT.set(snapshot);
    Ok(())
  }

  pub async fn load_config(environment: &Environment, config_file: Option<&str>) -> RecorderResult<AppConfig> {
    if config_file.is_some_and(|path| !Path::new(path).exists()) {
      return Err(crate::errors::RecorderError::InvalidConfiguration {
        message: "The selected configuration file does not exist".into(),
      });
    }
    let try_config_file_or_dirs = if config_file.is_some() { vec![config_file] } else { vec![Some(".")] };

    let allowed_extensions = &AppConfig::allowed_extension();
    let priority_suffix = &AppConfig::priority_suffix(environment);
    let convention_prefix = &AppConfig::config_prefix();

    let try_filenames = priority_suffix
      .iter()
      .flat_map(|ps| allowed_extensions.iter().map(move |ext| (format!("{convention_prefix}{ps}{ext}"), ext)))
      .collect_vec();

    let mut file = Figment::new();

    for try_config_file_or_dir in try_config_file_or_dirs.into_iter().flatten() {
      let try_config_file_or_dir_path = Path::new(try_config_file_or_dir);
      if try_config_file_or_dir_path.exists() {
        if try_config_file_or_dir_path.is_dir() {
          for (f, ext) in try_filenames.iter() {
            let p = try_config_file_or_dir_path.join(f);
            if p.exists() && p.is_file() {
              file = AppConfig::merge_provider_from_file(file, &p, ext)?;

              break;
            }
          }
        } else if let Some(ext) = try_config_file_or_dir_path.extension().and_then(|s| s.to_str())
          && try_config_file_or_dir_path.is_file()
        {
          file = AppConfig::merge_provider_from_file(file, try_config_file_or_dir_path, &format!(".{ext}"))?;

          break;
        }
      }
    }

    let file_tags = resolve::tags(&file)?;
    let snapshot = resolve::ENVIRONMENT.get_or_init(|| std::env::vars().collect());
    let fig = Self::merge_environment(Figment::from(AppConfig::default_provider()).merge(file), snapshot)?;
    let fig = Figment::from(resolve::interpolate(&fig, &file_tags, snapshot)?);
    for field in [
      "media.max_encoder_bytes",
      "media.jxl_distance",
      "media.jxl_quality",
      "media.jxl_effort",
      "media.jxl_speed",
      "media.jxl_encoder_path",
      "media.jxl_max_address_space_bytes",
    ] {
      if fig.find_value(field).is_ok() {
        return Err(resolve::invalid(
          field,
          "legacy libjxl/process option removed; migrate to the JPXL preset and shared admission budget",
        ));
      }
    }
    let mut app_config: AppConfig = fig
      .extract()
      .map_err(|error: figment::Error| crate::errors::RecorderError::InvalidConfiguration {
        message: format!("Invalid configuration at {} (values redacted)", error.path.join(".")),
      })?;

    app_config.resolve_database_urls();
    app_config.validate(environment)?;
    Ok(app_config)
  }
  pub fn resolve_database_urls(&mut self) {
    if self.auth.identity.database_url.is_empty() {
      self.auth.identity.database_url = self.database.uri.clone();
    }
    self.task.queue_database_uri.get_or_insert_with(|| self.database.uri.clone());
    if let ProviderAuthConfig::Bundled(provider) = &mut self.auth.provider
      && provider.database.url.is_empty()
    {
      provider.database.url = self.auth.identity.database_url.clone();
    }
  }

  pub(crate) fn validate(&self, environment: &Environment) -> RecorderResult<()> {
    let check_uri = |path: &str, value: &str| -> RecorderResult<()> {
      if value.is_empty() || value.chars().any(char::is_whitespace) || url::Url::parse(value).is_err() {
        return Err(resolve::invalid(path, "invalid database URI"));
      }
      Ok(())
    };
    check_uri("database.url", &self.database.uri)?;
    if let Some(uri) = &self.database.migration_database_uri {
      check_uri("database.migration.url", uri)?;
    }
    if let Some(uri) = &self.task.queue_database_uri {
      check_uri("scheduler.database.url", uri)?;
    }
    check_uri("auth.identity.database_url", &self.auth.identity.database_url)?;
    self
      .server
      .validate_external_base_url()
      .map_err(|message| resolve::invalid("server.external_base_url", message))?;
    self
      .auth
      .webui_policy()
      .map_err(|error| resolve::invalid("auth.webui.allowed_origins", &error.to_string()))?;
    if self.server.middlewares.cors.as_ref().is_some_and(|cors| cors.enable) {
      return Err(resolve::invalid(
        "server.middlewares.cors",
        "authentication owns CORS; configure auth.webui.allowed_origins instead of an independent CORS layer",
      ));
    }
    match &self.auth.provider {
      ProviderAuthConfig::Bundled(config) => {
        check_uri("auth.provider.database.url", &config.database.url)?;
        config
          .validate()
          .map_err(|_| resolve::invalid("auth.provider", "invalid bundled provider configuration"))?;
        if matches!(environment, Environment::Production) && url::Url::parse(&config.issuer).is_ok_and(|issuer| issuer.scheme() != "https") {
          return Err(resolve::invalid("auth.provider.issuer", "production requires an HTTPS issuer"));
        }
      }
      ProviderAuthConfig::Oidc(config) if config.issuer.trim().is_empty() => return Err(resolve::invalid("auth.provider.issuer", "empty issuer")),
      ProviderAuthConfig::Oidc(config) if config.browser.client_id.trim().is_empty() => {
        return Err(resolve::invalid("auth.provider.browser.client_id", "empty client id"));
      }
      _ => (),
    }
    for administrator in &self.auth.administrators {
      if administrator.subject.trim().is_empty() || url::Url::parse(&administrator.issuer).is_err() {
        return Err(resolve::invalid(
          "auth.administrators",
          "administrator requires an exact issuer and nonempty subject",
        ));
      }
    }
    if self.task.subscriber_task_concurrency == 0 || self.task.system_task_concurrency == 0 || self.task.cron_interval_duration.is_zero() {
      return Err(resolve::invalid("scheduler", "worker concurrency and cron interval must be positive"));
    }
    self.media.validate()
  }
}
