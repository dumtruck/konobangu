use std::fmt;

use bytes::Bytes;
use opendal::{Buffer, Operator, layers::LoggingLayer, services::Fs};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::StorageConfig;
use crate::errors::{RError, RResult};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageContentCategory {
    Image,
}

impl AsRef<str> for StorageContentCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Image => "image",
        }
    }
}

pub enum StorageStoredUrl {
    RelativePath { path: String },
    Absolute { url: Url },
}

impl AsRef<str> for StorageStoredUrl {
    fn as_ref(&self) -> &str {
        match &self {
            Self::Absolute { url } => url.as_str(),
            Self::RelativePath { path } => path,
        }
    }
}

impl fmt::Display for StorageStoredUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct StorageService {
    pub data_dir: String,
}

impl StorageService {
    pub async fn from_config(config: StorageConfig) -> RResult<Self> {
        Ok(Self {
            data_dir: config.data_dir.to_string(),
        })
    }

    pub fn get_fs(&self) -> Fs {
        Fs::default().root(&self.data_dir)
    }

    pub fn create_filename(extname: &str) -> String {
        format!("{}{}", Uuid::new_v4(), extname)
    }

    pub async fn store_object(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
        data: Bytes,
    ) -> Result<StorageStoredUrl, RError> {
        match content_category {
            StorageContentCategory::Image => {
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

                Ok(StorageStoredUrl::RelativePath {
                    path: fullname.to_string(),
                })
            }
        }
    }

    pub async fn exists_object(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
    ) -> Result<Option<StorageStoredUrl>, RError> {
        match content_category {
            StorageContentCategory::Image => {
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
                    Ok(Some(StorageStoredUrl::RelativePath {
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
        content_category: StorageContentCategory,
        subscriber_pid: &str,
        bucket: Option<&str>,
        filename: &str,
    ) -> color_eyre::eyre::Result<Buffer> {
        match content_category {
            StorageContentCategory::Image => {
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
