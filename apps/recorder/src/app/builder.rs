use clap::{Parser, command};

use super::{AppContext, core::App, env::Environment};
use crate::{app::config::AppConfig, errors::RecorderResult};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct MainCliArgs {
    /// Explicit config file path
    #[arg(short, long)]
    config_file: Option<String>,

    /// Explicit dotenv file path
    #[arg(short, long)]
    dotenv_file: Option<String>,

    /// Explicit working dir
    #[arg(short, long)]
    working_dir: Option<String>,

    /// Explicit environment
    #[arg(short, long)]
    environment: Option<Environment>,
}

pub struct AppBuilder {
    dotenv_file: Option<String>,
    config_file: Option<String>,
    working_dir: String,
    environment: Environment,
}

impl AppBuilder {
    pub async fn from_main_cli(environment: Option<Environment>) -> RecorderResult<Self> {
        let args = MainCliArgs::parse();

        let environment = environment.unwrap_or_else(|| {
            args.environment.unwrap_or({
                if cfg!(test) {
                    Environment::Testing
                } else if cfg!(debug_assertions) {
                    Environment::Development
                } else {
                    Environment::Production
                }
            })
        });

        let mut builder = Self::default();

        if let Some(working_dir) = args.working_dir {
            builder = builder.working_dir(working_dir);
        }
        if matches!(
            &environment,
            Environment::Testing | Environment::Development
        ) {
            builder = builder.working_dir_from_manifest_dir();
        }

        builder = builder
            .config_file(args.config_file)
            .dotenv_file(args.dotenv_file)
            .environment(environment);

        Ok(builder)
    }

    pub async fn build(self) -> RecorderResult<App> {
        self.load_env().await?;

        let config = self.load_config().await?;

        let app_context =
            AppContext::new(self.environment.clone(), config, self.working_dir.clone()).await?;

        Ok(App {
            context: app_context,
            builder: self,
        })
    }

    pub async fn load_env(&self) -> RecorderResult<()> {
        AppConfig::load_dotenv(
            &self.environment,
            &self.working_dir,
            self.dotenv_file.as_deref(),
        )
        .await?;
        Ok(())
    }

    pub async fn load_config(&self) -> RecorderResult<AppConfig> {
        let config = AppConfig::load_config(
            &self.environment,
            &self.working_dir,
            self.config_file.as_deref(),
        )
        .await?;
        Ok(config)
    }

    pub fn working_dir(self, working_dir: String) -> Self {
        let mut ret = self;
        ret.working_dir = working_dir;
        ret
    }

    pub fn environment(self, environment: Environment) -> Self {
        let mut ret = self;
        ret.environment = environment;
        ret
    }

    pub fn config_file(self, config_file: Option<String>) -> Self {
        let mut ret = self;
        ret.config_file = config_file;
        ret
    }

    pub fn dotenv_file(self, dotenv_file: Option<String>) -> Self {
        let mut ret = self;
        ret.dotenv_file = dotenv_file;
        ret
    }

    pub fn working_dir_from_manifest_dir(self) -> Self {
        let manifest_dir = if cfg!(debug_assertions) || cfg!(test) {
            env!("CARGO_MANIFEST_DIR")
        } else {
            "./apps/recorder"
        };
        self.working_dir(manifest_dir.to_string())
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            environment: Environment::Production,
            dotenv_file: None,
            config_file: None,
            working_dir: String::from("."),
        }
    }
}
