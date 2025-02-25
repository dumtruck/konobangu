use std::fmt;

use async_trait::async_trait;
use bytes::Bytes;
use loco_rs::app::{AppContext, Initializer};
use once_cell::sync::OnceCell;
use opendal::{Buffer, Operator, layers::LoggingLayer, services::Fs};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::AppDalConfig;
use crate::{app::App, config::AppConfigExt, errors::RecorderError};

// TODO: wait app-context-trait to integrate
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DalContentCategory {
    Image,
}

impl AsRef<str> for DalContentCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Image => "image",
        }
    }
}

static APP_DAL_CLIENT: OnceCell<AppDalClient> = OnceCell::new();

pub enum DalStoredUrl {
    RelativePath { path: String },
    Absolute { url: Url },
}

impl AsRef<str> for DalStoredUrl {
    fn as_ref(&self) -> &str {
        match &self {
            Self::Absolute { url } => url.as_str(),
            Self::RelativePath { path } => path,
        }
    }
}

impl fmt::Display for DalStoredUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct AppDalClient {
    pub data_dir: String,
}

impl AppDalClient {
    pub fn new(config: AppDalConfig) -> Self {
        Self {
            data_dir: App::get_working_root()
                .join(config.data_dir.as_deref().unwrap_or("./data"))
                .to_string(),
        }
    }

    pub fn app_instance() -> &'static AppDalClient {
        APP_DAL_CLIENT
            .get()
            .expect("AppDalClient is not initialized")
    }

    pub fn get_fs(&self) -> Fs {
        Fs::default().root(&self.data_dir)
    }

    pub fn create_filename(extname: &str) -> String {
        format!("{}{}", Uuid::new_v4(), extname)
    }

    pub async fn store_object(
        &self,
        content_category: DalContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
        data: Bytes,
    ) -> Result<DalStoredUrl, RecorderError> {
        match content_category {
            DalContentCategory::Image => {
                let fullname = [
                    &subscriber_id.to_string(),
                    content_category.as_ref(),
                    bucket.unwrap_or_default(),
                    filename,
                ]
                .into_iter()
                .map(Path::new)
                .collect::<PathBuf>();

                let fs_op = Operator::new(self.get_fs())?
                    .layer(LoggingLayer::default())
                    .finish();

                if let Some(dirname) = fullname.parent() {
                    let dirname = dirname.join("/");
                    fs_op.create_dir(dirname.as_str()).await?;
                }

                fs_op.write(fullname.as_str(), data).await?;

                Ok(DalStoredUrl::RelativePath {
                    path: fullname.to_string(),
                })
            }
        }
    }

    pub async fn exists_object(
        &self,
        content_category: DalContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
    ) -> Result<Option<DalStoredUrl>, RecorderError> {
        match content_category {
            DalContentCategory::Image => {
                let fullname = [
                    &subscriber_id.to_string(),
                    content_category.as_ref(),
                    bucket.unwrap_or_default(),
                    filename,
                ]
                .into_iter()
                .map(Path::new)
                .collect::<PathBuf>();

                let fs_op = Operator::new(self.get_fs())?
                    .layer(LoggingLayer::default())
                    .finish();

                if fs_op.exists(fullname.as_str()).await? {
                    Ok(Some(DalStoredUrl::RelativePath {
                        path: fullname.to_string(),
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn load_object(
        &self,
        content_category: DalContentCategory,
        subscriber_pid: &str,
        bucket: Option<&str>,
        filename: &str,
    ) -> color_eyre::eyre::Result<Buffer> {
        match content_category {
            DalContentCategory::Image => {
                let fullname = [
                    subscriber_pid,
                    content_category.as_ref(),
                    bucket.unwrap_or_default(),
                    filename,
                ]
                .into_iter()
                .map(Path::new)
                .collect::<PathBuf>();

                let fs_op = Operator::new(self.get_fs())?
                    .layer(LoggingLayer::default())
                    .finish();

                let data = fs_op.read(fullname.as_str()).await?;

                Ok(data)
            }
        }
    }
}

pub struct AppDalInitalizer;

#[async_trait]
impl Initializer for AppDalInitalizer {
    fn name(&self) -> String {
        String::from("AppDalInitalizer")
    }

    async fn before_run(&self, app_context: &AppContext) -> loco_rs::Result<()> {
        let config = &app_context.config;
        let app_dal_conf = config.get_app_conf()?.dal;

        APP_DAL_CLIENT.get_or_init(|| AppDalClient::new(app_dal_conf));

        Ok(())
    }
}
