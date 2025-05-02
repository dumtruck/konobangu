use std::fmt;

use bytes::Bytes;
use opendal::{Buffer, Operator, layers::LoggingLayer};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;

use super::StorageConfig;
use crate::errors::app_error::RecorderResult;

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

#[async_trait::async_trait]
pub trait StorageServiceTrait: Sync {
    fn get_operator(&self) -> RecorderResult<Operator>;

    fn get_fullname(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
    ) -> PathBuf {
        [
            &subscriber_id.to_string(),
            content_category.as_ref(),
            bucket.unwrap_or_default(),
            filename,
        ]
        .into_iter()
        .map(Path::new)
        .collect::<PathBuf>()
    }
    async fn store_object(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
        data: Bytes,
    ) -> RecorderResult<StorageStoredUrl> {
        let fullname = self.get_fullname(content_category, subscriber_id, bucket, filename);

        let operator = self.get_operator()?;

        if let Some(dirname) = fullname.parent() {
            let dirname = dirname.join("/");
            operator.create_dir(dirname.as_str()).await?;
        }

        operator.write(fullname.as_str(), data).await?;

        Ok(StorageStoredUrl::RelativePath {
            path: fullname.to_string(),
        })
    }

    async fn exists_object(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
    ) -> RecorderResult<Option<StorageStoredUrl>> {
        let fullname = self.get_fullname(content_category, subscriber_id, bucket, filename);

        let operator = self.get_operator()?;

        if operator.exists(fullname.as_str()).await? {
            Ok(Some(StorageStoredUrl::RelativePath {
                path: fullname.to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn load_object(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: Option<&str>,
        filename: &str,
    ) -> RecorderResult<Buffer> {
        let fullname = self.get_fullname(content_category, subscriber_id, bucket, filename);

        let operator = self.get_operator()?;

        let data = operator.read(fullname.as_str()).await?;

        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct StorageService {
    pub data_dir: String,
}

impl StorageService {
    pub async fn from_config(config: StorageConfig) -> RecorderResult<Self> {
        Ok(Self {
            data_dir: config.data_dir.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl StorageServiceTrait for StorageService {
    fn get_operator(&self) -> RecorderResult<Operator> {
        let fs_op = Operator::new(opendal::services::Fs::default().root(&self.data_dir))?
            .layer(LoggingLayer::default())
            .finish();

        Ok(fs_op)
    }
}
