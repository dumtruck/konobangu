use std::{path::Path, sync::Arc};

use figment::Figment;
use itertools::Itertools;

use super::{core::App, env::Enviornment};
use crate::{
    app::{config::AppConfig, context::create_context, router::create_router},
    errors::RResult,
};

pub struct AppBuilder {
    dotenv_file: Option<String>,
    config_file: Option<String>,
    working_dir: String,
    enviornment: Enviornment,
}

impl AppBuilder {
    pub async fn load_dotenv(&self) -> RResult<()> {
        let try_dotenv_file_or_dirs = if self.dotenv_file.is_some() {
            vec![self.dotenv_file.as_deref()]
        } else {
            vec![Some(&self.working_dir as &str)]
        };

        let priority_suffix = &AppConfig::priority_suffix(&self.enviornment);
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
                            dotenv::from_path(p)?;
                            break;
                        }
                    }
                } else if try_dotenv_file_or_dir_path.is_file() {
                    dotenv::from_path(try_dotenv_file_or_dir_path)?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn build_config(&self) -> RResult<AppConfig> {
        let try_config_file_or_dirs = if self.config_file.is_some() {
            vec![self.config_file.as_deref()]
        } else {
            vec![Some(&self.working_dir as &str)]
        };

        let allowed_extensions = &AppConfig::allowed_extension();
        let priority_suffix = &AppConfig::priority_suffix(&self.enviornment);
        let convention_prefix = &AppConfig::config_prefix();

        let try_filenames = priority_suffix
            .iter()
            .flat_map(|ps| {
                allowed_extensions
                    .iter()
                    .map(move |ext| (format!("{}{}{}", convention_prefix, ps, ext), ext))
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
                    break;
                }
            }
        }

        let app_config: AppConfig = fig.extract()?;

        Ok(app_config)
    }

    pub async fn build(self) -> RResult<App> {
        let _app_name = env!("CARGO_CRATE_NAME");

        let _app_version = format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        );

        self.load_dotenv().await?;

        let config = self.build_config().await?;

        let app_context = Arc::new(create_context(config).await?);

        let router = create_router(app_context.clone()).await?;

        Ok(App {
            context: app_context,
            router,
            builder: self,
        })
    }

    pub fn set_working_dir(self, working_dir: String) -> Self {
        let mut ret = self;
        ret.working_dir = working_dir;
        ret
    }

    pub fn set_working_dir_to_manifest_dir(self) -> Self {
        let manifest_dir = if cfg!(debug_assertions) {
            env!("CARGO_MANIFEST_DIR")
        } else {
            "./apps/recorder"
        };
        self.set_working_dir(manifest_dir.to_string())
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            enviornment: Enviornment::Production,
            dotenv_file: None,
            config_file: None,
            working_dir: String::from("."),
        }
    }
}
