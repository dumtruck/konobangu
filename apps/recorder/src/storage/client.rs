use std::fmt;

use bytes::Bytes;
use opendal::{Buffer, Metadata, Operator, Reader, Writer, layers::LoggingLayer};
use quirks_path::PathBuf;
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

    pub fn get_operator(&self) -> Result<Operator, opendal::Error> {
        let op = if cfg!(test) {
            Operator::new(opendal::services::Memory::default())?
                .layer(LoggingLayer::default())
                .finish()
        } else {
            Operator::new(opendal::services::Fs::default().root(&self.data_dir))?
                .layer(LoggingLayer::default())
                .finish()
        };

        Ok(op)
    }

    pub fn build_subscriber_path(&self, subscriber_id: i32, path: &str) -> PathBuf {
        let mut p = PathBuf::from("/subscribers");
        p.push(subscriber_id.to_string());
        p.push(path);
        p
    }

    pub fn build_subscriber_object_path(
        &self,
        content_category: StorageContentCategory,
        subscriber_id: i32,
        bucket: &str,
        object_name: &str,
    ) -> PathBuf {
        self.build_subscriber_path(
            subscriber_id,
            &format!("{}/{}/{}", content_category.as_ref(), bucket, object_name),
        )
    }

    pub async fn write<P: Into<PathBuf> + Send>(
        &self,
        path: P,
        data: Bytes,
    ) -> Result<StorageStoredUrl, opendal::Error> {
        let operator = self.get_operator()?;

        let path = path.into();

        if let Some(dirname) = path.parent() {
            let dirname = dirname.join("/");
            operator.create_dir(dirname.as_str()).await?;
        }

        operator.write(path.as_str(), data).await?;

        Ok(StorageStoredUrl::RelativePath {
            path: path.to_string(),
        })
    }

    pub async fn exists<P: ToString + Send>(
        &self,
        path: P,
    ) -> Result<Option<StorageStoredUrl>, opendal::Error> {
        let operator = self.get_operator()?;

        let path = path.to_string();

        if operator.exists(&path).await? {
            Ok(Some(StorageStoredUrl::RelativePath { path }))
        } else {
            Ok(None)
        }
    }

    pub async fn read(&self, path: impl AsRef<str>) -> Result<Buffer, opendal::Error> {
        let operator = self.get_operator()?;

        let data = operator.read(path.as_ref()).await?;

        Ok(data)
    }

    pub async fn reader(&self, path: impl AsRef<str>) -> Result<Reader, opendal::Error> {
        let operator = self.get_operator()?;

        let reader = operator.reader(path.as_ref()).await?;

        Ok(reader)
    }

    pub async fn writer(&self, path: impl AsRef<str>) -> Result<Writer, opendal::Error> {
        let operator = self.get_operator()?;

        let writer = operator.writer(path.as_ref()).await?;

        Ok(writer)
    }

    pub async fn stat(&self, path: impl AsRef<str>) -> Result<Metadata, opendal::Error> {
        let operator = self.get_operator()?;

        let metadata = operator.stat(path.as_ref()).await?;

        Ok(metadata)
    }
}
