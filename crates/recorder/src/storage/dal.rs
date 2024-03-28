use std::fmt::Display;

use bytes::Bytes;
use opendal::{layers::LoggingLayer, services, Operator};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::config::AppDalConf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DalContentType {
    Poster,
}

impl AsRef<str> for DalContentType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Poster => "poster",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DalContext {
    pub config: AppDalConf,
}

pub enum DalStoredUrl {
    RelativePath { path: String },
    Absolute { url: Url },
}

impl DalStoredUrl {
    pub fn as_str(&self) -> &str {
        match self {
            Self::RelativePath { path } => path.as_str(),
            Self::Absolute { url } => url.as_str(),
        }
    }
}

impl AsRef<str> for DalStoredUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for DalStoredUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str().to_string())
    }
}

impl DalContext {
    pub fn new(dal_conf: AppDalConf) -> Self {
        Self { config: dal_conf }
    }

    pub async fn store_blob(
        &self,
        content_category: DalContentType,
        extname: &str,
        data: Bytes,
        subscriber_pid: &str,
    ) -> eyre::Result<DalStoredUrl> {
        let basename = format!("{}{}", Uuid::new_v4(), extname);
        let mut dirname = [subscriber_pid, content_category.as_ref()]
            .into_iter()
            .map(Path::new)
            .collect::<PathBuf>();

        let mut fs_builder = services::Fs::default();
        fs_builder.root(self.config.fs_root.as_str());

        let fs_op = Operator::new(fs_builder)?
            .layer(LoggingLayer::default())
            .finish();

        fs_op.create_dir(dirname.as_str()).await?;

        let fullname = {
            dirname.push(basename);
            dirname
        };

        fs_op.write_with(fullname.as_str(), data).await?;

        Ok(DalStoredUrl::RelativePath {
            path: fullname.to_string(),
        })
    }
}
